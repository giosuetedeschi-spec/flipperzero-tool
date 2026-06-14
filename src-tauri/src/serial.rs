//! Serial communication module for Flipper Zero.
//!
//! Handles:
//!   - USB serial port discovery (VID:PID auto-detect)
//!   - Protobuf varint framing for RPC messages
//!   - Flipper CLI command protocol (for file operations)
//!   - Thread-safe connection state via Arc<Mutex<...>>

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use super::errors::AppError;

// ---------------------------------------------------------------------------
// Port discovery
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortInfo {
    pub port_name: String,
    pub port_type: String,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
}

/// List all available serial ports.
pub fn list_ports() -> Result<Vec<PortInfo>, AppError> {
    let ports = serialport::available_ports()
        .map_err(|e| AppError::SerialError(format!("Cannot list ports: {}", e)))?;

    Ok(ports
        .into_iter()
        .map(|p| {
            let (port_type, manufacturer, product, serial_number) = match &p.port_type {
                serialport::SerialPortType::UsbPort(usb) => (
                    "usb".to_string(),
                    usb.manufacturer.clone(),
                    usb.product.clone(),
                    usb.serial_number.clone(),
                ),
                serialport::SerialPortType::BluetoothPort => ("bluetooth".to_string(), None, None, None),
                serialport::SerialPortType::PciPort => ("pci".to_string(), None, None, None),
            };
            PortInfo {
                port_name: p.port_name.clone(),
                port_type,
                manufacturer,
                product,
                serial_number,
            }
        })
        .collect())
}

/// Find Flipper Zero by VID:PID (0483:5740).
pub fn find_flipper() -> Result<Option<PortInfo>, AppError> {
    let ports = list_ports()?;
    for port in &ports {
        if let serialport::SerialPortType::UsbPort(_) = serialport::new(&port.port_name, 0).port_type {
            // Already filtered in list_ports below
        }
    }
    // Use serialport to check VID:PID
    let ports_raw = serialport::available_ports()
        .map_err(|e| AppError::SerialError(format!("Cannot scan ports: {}", e)))?;

    for p in &ports_raw {
        if let serialport::SerialPortType::UsbPort(usb) = &p.port_type {
            if usb.vid == 0x0483 && usb.pid == 0x5740 {
                return Ok(Some(PortInfo {
                    port_name: p.port_name.clone(),
                    port_type: "usb".to_string(),
                    manufacturer: usb.manufacturer.clone(),
                    product: usb.product.clone(),
                    serial_number: usb.serial_number.clone(),
                }));
            }
        }
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// Protobuf helpers (manual, no prost dependency at runtime)
// ---------------------------------------------------------------------------

/// Encode a u64 as a protobuf varint (LEB128 unsigned).
pub fn encode_varint(buf: &mut Vec<u8>, value: u64) {
    let mut v = value;
    while v >= 0x80 {
        buf.push((v as u8) | 0x80);
        v >>= 7;
    }
    buf.push(v as u8);
}

/// Read a varint from a byte reader with timeout.
pub fn read_varint<R: Read>(reader: &mut R, timeout: Duration) -> Result<u64, AppError> {
    let mut result: u64 = 0;
    let mut shift = 0u32;

    let mut buf = [0u8; 1];
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            return Err(AppError::SerialError("Varint read timeout".to_string()));
        }

        reader
            .read_exact(&mut buf)
            .map_err(|e| AppError::SerialError(format!("Varint read error: {}", e)))?;

        let byte = buf[0];
        result |= ((byte & 0x7F) as u64) << shift;

        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(AppError::SerialError("Varint overflow".to_string()));
        }
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Base64 encoding (for binary data over CLI)
// ---------------------------------------------------------------------------

const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encode bytes to base64 string (standard, no line wrapping).
pub fn base64_encode(data: &[u8]) -> String {
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    let chunks = data.chunks_exact(3);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let n = (chunk[0] as u32) << 16 | (chunk[1] as u32) << 8 | chunk[2] as u32;
        result.push(B64_CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(B64_CHARS[((n >> 12) & 0x3F) as usize] as char);
        result.push(B64_CHARS[((n >> 6) & 0x3F) as usize] as char);
        result.push(B64_CHARS[(n & 0x3F) as usize] as char);
    }

    match remainder.len() {
        1 => {
            let n = (remainder[0] as u32) << 16;
            result.push(B64_CHARS[((n >> 18) & 0x3F) as usize] as char);
            result.push(B64_CHARS[((n >> 12) & 0x3F) as usize] as char);
            result.push('=');
            result.push('=');
        }
        2 => {
            let n = (remainder[0] as u32) << 16 | (remainder[1] as u32) << 8;
            result.push(B64_CHARS[((n >> 18) & 0x3F) as usize] as char);
            result.push(B64_CHARS[((n >> 12) & 0x3F) as usize] as char);
            result.push(B64_CHARS[((n >> 6) & 0x3F) as usize] as char);
            result.push('=');
        }
        _ => {}
    }

    result
}

// ---------------------------------------------------------------------------
// Flipper CLI output parsing
// ---------------------------------------------------------------------------

/// Parse the output of Flipper's `storage list` CLI command.
///
/// Expected format per line:
///   [D] dirname
///   [F] filename size
///
/// Returns entries sorted: directories first, then files.
pub fn parse_list_output(base_path: &str, output: &str) -> Result<Vec<super::commands::FileInfo>, AppError> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    let base = if base_path.ends_with('/') {
        base_path.trim_end_matches('/').to_string()
    } else {
        base_path.to_string()
    };

    for line in output.lines() {
        let line = line.trim();
        if line.len() < 4 {
            continue;
        }

        if let Some(rest) = line.strip_prefix("[D] ") {
            let name = rest.trim().to_string();
            if name.is_empty() {
                continue;
            }
            dirs.push(super::commands::FileInfo {
                path: format!("{}/{}", base, name),
                name,
                size: 0,
                is_dir: true,
                modified: None,
            });
        } else if let Some(rest) = line.strip_prefix("[F] ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let size = parts[1].parse().unwrap_or(0);
                files.push(super::commands::FileInfo {
                    path: format!("{}/{}", base, name),
                    name,
                    size,
                    is_dir: false,
                    modified: None,
                });
            }
        }
    }

    let mut result = dirs;
    result.extend(files);
    Ok(result)
}

// ---------------------------------------------------------------------------
// Flipper connection state (thread-safe)
// ---------------------------------------------------------------------------

pub struct FlipperConnection {
    pub port_name: String,
    pub baud_rate: u32,
    pub connected: bool,
    next_seq: u32,
}

impl FlipperConnection {
    pub fn new() -> Self {
        Self {
            port_name: String::new(),
            baud_rate: 115200,
            connected: false,
        }
    }
}

/// Thread-safe shared state for the serial connection.
pub type FlipperState = Arc<Mutex<Option<FlipperConnection>>>;

/// Create a new FlipperState.
pub fn new_state() -> FlipperState {
    Arc::new(Mutex::new(None))
}

/// Check if currently connected.
pub fn is_connected(state: &FlipperState) -> bool {
    state
        .lock()
        .map(|guard| guard.as_ref().map_or(false, |c| c.connected))
        .unwrap_or(false)
}

/// Connect to a serial port.
pub fn connect(state: &FlipperState, port: &str) -> Result<bool, AppError> {
    let mut guard = state.lock().map_err(|_| AppError::SerialError("Mutex poisoned".into()))?;
    *guard = Some(FlipperConnection {
        port_name: port.to_string(),
        baud_rate: 115200,
        connected: true,
    });
    Ok(true)
}

/// Disconnect from the current port.
pub fn disconnect(state: &FlipperState) -> Result<bool, AppError> {
    let mut guard = state.lock().map_err(|_| AppError::SerialError("Mutex poisoned".into()))?;
    if let Some(ref mut conn) = *guard {
        conn.connected = false;
    }
    *guard = None;
    Ok(true)
}

/// Read a text file from the Flipper via CLI.
pub fn read_file_text(state: &FlipperState, path: &str) -> Result<String, AppError> {
    let _guard = state.lock().map_err(|_| AppError::SerialError("Mutex poisoned".into()))?;
    // In a real implementation, this would send a CLI command and read the response.
    // For now, return a placeholder that indicates the command was received.
    Err(AppError::SerialError(format!(
        "read_file_text not yet implemented for path: {}",
        path
    )))
}

/// Write a text file to the Flipper via CLI.
pub fn write_file_text(state: &FlipperState, path: &str, _data: &str) -> Result<bool, AppError> {
    let _guard = state.lock().map_err(|_| AppError::SerialError("Mutex poisoned".into()))?;
    Err(AppError::SerialError(format!(
        "write_file_text not yet implemented for path: {}",
        path
    )))
}

/// List directory on the Flipper via CLI.
pub fn list_dir(state: &FlipperState, path: &str) -> Result<Vec<super::commands::FileInfo>, AppError> {
    let _guard = state.lock().map_err(|_| AppError::SerialError("Mutex poisoned".into()))?;
    // In a real implementation, send `storage list <path>` and parse output.
    // For now, return empty.
    parse_list_output(path, "")
}

/// Delete a file or directory on the Flipper.
pub fn delete_path(state: &FlipperState, path: &str) -> Result<bool, AppError> {
    let _guard = state.lock().map_err(|_| AppError::SerialError("Mutex poisoned".into()))?;
    Err(AppError::SerialError(format!(
        "delete_path not yet implemented for path: {}",
        path
    )))
}

/// Create a directory on the Flipper.
pub fn mkdir_path(state: &FlipperState, path: &str) -> Result<bool, AppError> {
    let _guard = state.lock().map_err(|_| AppError::SerialError("Mutex poisoned".into()))?;
    Err(AppError::SerialError(format!(
        "mkdir_path not yet implemented for path: {}",
        path
    )))
}

/// Get file stats from the Flipper.
pub fn stat_path(state: &FlipperState, path: &str) -> Result<super::commands::FileInfo, AppError> {
    let _guard = state.lock().map_err(|_| AppError::SerialError("Mutex poisoned".into()))?;
    Err(AppError::SerialError(format!(
        "stat_path not yet implemented for path: {}",
        path
    )))
}

/// Auto-detect and connect to Flipper Zero.
pub fn autodetect_connect(state: &FlipperState) -> Result<bool, AppError> {
    match find_flipper()? {
        Some(port) => connect(state, &port.port_name),
        None => Err(AppError::SerialError(
            "Flipper Zero not found (VID:PID 0483:5740)".to_string(),
        )),
    }
}


// ---------------------------------------------------------------------------
// Sequence management for RPC
// ---------------------------------------------------------------------------

impl FlipperConnection {
    pub fn next_sequence(&mut self) -> u32 {
        self.next_seq += 1;
        if self.next_seq == 0 { self.next_seq = 1; }
        self.next_seq
    }

    pub fn current_sequence(&self) -> u32 {
        self.next_seq
    }

    /// Write an RPC message with varint length prefix.
    pub fn write_rpc(&mut self, data: &[u8]) -> Result<(), AppError> {
        if !self.connected {
            return Err(AppError::SerialError("Not connected".to_string()));
        }

        // Encode length as varint
        let mut frame = encode_varint(data.len() as u64);
        frame.extend_from_slice(data);

        // Write to serial port
        self.port.as_mut().map(|p| {
            p.write_all(&frame).map_err(|e| AppError::SerialError(format!("Write error: {}", e)))
        }).unwrap_or(Err(AppError::SerialError("No port".to_string())))?;

        Ok(())
    }

    /// Read an RPC message (varint length prefix + data).
    pub fn read_rpc(&mut self, timeout_ms: u64) -> Result<Vec<u8>, AppError> {
        if !self.connected {
            return Err(AppError::SerialError("Not connected".to_string()));
        }

        let port = self.port.as_mut()
            .ok_or_else(|| AppError::SerialError("No port".to_string()))?;

        // Read varint length
        let mut len_buf = Vec::new();
        let mut byte = [0u8; 1];
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

        loop {
            if std::time::Instant::now() > deadline {
                return Err(AppError::SerialError("RPC read timeout".to_string()));
            }
            match port.read(&mut byte) {
                Ok(0) => continue,
                Ok(_) => {
                    len_buf.push(byte[0]);
                    if byte[0] & 0x80 == 0 {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(e) => return Err(AppError::SerialError(format!("Read error: {}", e))),
            }
        }

        let (length, _) = decode_varint(&len_buf)
            .map_err(|e| AppError::SerialError(format!("Varint decode: {}", e)))?;

        // Read payload
        let mut payload = vec![0u8; length as usize];
        port.read_exact(&mut payload)
            .map_err(|e| AppError::SerialError(format!("Payload read: {}", e)))?;

        Ok(payload)
    }
}
