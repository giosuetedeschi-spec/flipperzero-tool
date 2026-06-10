//! Serial communication module for Flipper Zero.
//!
//! Architecture:
//!   proto.rs        - Protobuf varint framing (encode/decode)
//!   connection.rs   - Connection management (open/send/recv)
//!   mod.rs          - Public API (re-exports)
//!
//! The Flipper Zero uses a binary protocol over USB serial:
//!   1. Protobuf messages encoded with varint length prefix
//!   2. 115200 baud, 3-second timeout
//!   3. VID:PID 0x0483:0x5740

pub mod proto;
pub mod connection;

pub use connection::{FlipperConnection, FlipperState, is_flipper_port};
pub use proto::{FLIPPER_VID, FLIPPER_PID, FLIPPER_BAUD, FLIPPER_TIMEOUT};

use std::sync::Mutex;
use std::sync::Arc;
use super::errors::AppError;
use super::commands::FileInfo;

/// List available serial ports.
pub fn list_ports() -> Result<Vec<super::PortInfo>, AppError> {
    let ports = serialport::available_ports()
        .map_err(|e| AppError::SerialError(e.to_string()))?;

    Ok(ports.into_iter().map(|p| super::PortInfo {
        name: p.port_name.clone(),
        port_type: format!("{:?}", p.port_type),
        description: match p.port_type {
            serialport::SerialPortType::UsbPort(info) => {
                Some(format!("{:04x}:{:04x} vid={:?} pid={:?}",
                    info.vid, info.pid,
                    info.manufacturer.as_deref().unwrap_or(""),
                    info.product.as_deref().unwrap_or("")))
            }
            _ => None,
        },
    }).collect())
}

/// Create a new FlipperState for use with tauri::State.
pub fn new_state() -> FlipperState {
    Arc::new(Mutex::new(None))
}

/// Connect to a Flipper on the given port.
pub fn connect(state: &FlipperState, port: &str) -> Result<bool, AppError> {
    let mut conn = FlipperConnection::open(port)?;
    let mut guard = state.lock()
        .map_err(|_| AppError::SerialError("Mutex poisoned".to_string()))?;
    *guard = Some(conn);
    Ok(true)
}

/// Disconnect from the Flipper.
pub fn disconnect(state: &FlipperState) -> Result<bool, AppError> {
    let mut guard = state.lock()
        .map_err(|_| AppError::SerialError("Mutex poisoned".to_string()))?;
    *guard = None;
    Ok(true)
}

/// Check if currently connected.
pub fn is_connected(state: &FlipperState) -> bool {
    state.lock()
        .map(|g| g.is_some())
        .unwrap_or(false)
}

/// List files on the Flipper via serial.
pub fn list_dir(state: &FlipperState, path: &str) -> Result<Vec<FileInfo>, AppError> {
    let mut guard = state.lock()
        .map_err(|_| AppError::SerialError("Mutex poisoned".to_string()))?;
    let conn = guard.as_mut()
        .ok_or_else(|| AppError::SerialError("Not connected".to_string()))?;

    // For now, fall back to CLI-based implementation
    // TODO: Replace with protobuf RPC once proto_gen types are available
    let cmd = format!("storage list {}", path);
    let output = execute_cli_command(conn, &cmd)?;
    parse_list_output(path, &output)
}

/// Read a file from the Flipper.
pub fn read_file(state: &FlipperState, path: &str) -> Result<Vec<u8>, AppError> {
    let mut guard = state.lock()
        .map_err(|_| AppError::SerialError("Mutex poisoned".to_string()))?;
    let conn = guard.as_mut()
        .ok_or_else(|| AppError::SerialError("Not connected".to_string()))?;

    let cmd = format!("storage read {}", path);
    let output = execute_cli_command(conn, &cmd)?;
    Ok(output.into_bytes())
}

/// Read a file as text.
pub fn read_file_text(state: &FlipperState, path: &str) -> Result<String, AppError> {
    let bytes = read_file(state, path)?;
    String::from_utf8(bytes)
        .map_err(|e| AppError::ParseError(format!("Not valid UTF-8: {}", e)))
}

/// Write a file to the Flipper.
pub fn write_file(state: &FlipperState, path: &str, data: &[u8]) -> Result<bool, AppError> {
    let mut guard = state.lock()
        .map_err(|_| AppError::SerialError("Mutex poisoned".to_string()))?;
    let conn = guard.as_mut()
        .ok_or_else(|| AppError::SerialError("Not connected".to_string()))?;

    use super::proto::encode_frame;
    let encoded = base64_encode(data);
    let cmd = format!("storage write {} {}", path, encoded);
    execute_cli_command(conn, &cmd)?;
    Ok(true)
}

/// Write text to a file.
pub fn write_file_text(state: &FlipperState, path: &str, content: &str) -> Result<bool, AppError> {
    write_file(state, path, content.as_bytes())
}

// --- CLI fallback (will be replaced by protobuf RPC) ---

fn execute_cli_command(port: &mut FlipperConnection, cmd: &str) -> Result<String, AppError> {
    use std::io::Write;
    let cmd_line = format!("{}
", cmd);
    port.port.write_all(cmd_line.as_bytes())
        .map_err(|e| AppError::SerialError(format!("Write error: {}", e)))?;
    port.port.flush()
        .map_err(|e| AppError::SerialError(format!("Flush error: {}", e)))?;

    let mut response = String::new();
    let mut buf = [0u8; 1024];
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(3);

    loop {
        if start.elapsed() > timeout {
            return Err(AppError::SerialError("Timeout".to_string()));
        }
        match (&mut port.port).read(&mut buf) {
            Ok(0) => {
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
            Ok(n) => {
                response.push_str(&String::from_utf8_lossy(&buf[..n]));
                if response.contains(">:") {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                return Err(AppError::SerialError("Timeout".to_string()));
            }
            Err(e) => return Err(AppError::SerialError(format!("Read error: {}", e))),
        }
    }

    // Strip prompt
    if let Some(idx) = response.rfind(">:") {
        response.truncate(idx);
    }
    Ok(response.trim().to_string())
}

fn parse_list_output(path: &str, output: &str) -> Result<Vec<FileInfo>, AppError> {
    let mut entries = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if let Some(rest) = line.strip_prefix("[F] ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if !parts.is_empty() {
                entries.push(FileInfo {
                    path: format!("{}/{}", path.trim_end_matches('/'), parts[0]),
                    name: parts[0].to_string(),
                    size: parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
                    is_dir: false,
                    modified: None,
                });
            }
        } else if let Some(rest) = line.strip_prefix("[D] ") {
            entries.push(FileInfo {
                path: format!("{}/{}", path.trim_end_matches('/'), rest.trim()),
                name: rest.trim().to_string(),
                size: 0,
                is_dir: true,
                modified: None,
            });
        }
    }
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
    Ok(entries)
}

fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let n = (b0 as u32) << 16 | (b1 as u32) << 8 | (b2 as u32);
        result.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char); }
        else { result.push('='); }
        if chunk.len() > 2 { result.push(ALPHABET[(n & 0x3F) as usize] as char); }
        else { result.push('='); }
    }
    result
}
