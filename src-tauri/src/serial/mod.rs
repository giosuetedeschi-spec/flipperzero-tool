//! Serial communication module for Flipper Zero.
//!
//! Provides USB port discovery (VID:PID auto-detect), CLI-based file
//! operations over the serial port, and varint framing for future
//! Protobuf RPC support (see `proto_bus.rs`, which owns the actual
//! binary RPC framing and will need a real port handle to transmit).

pub use super::commands::FileInfo;
use super::errors::AppError;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const FLIPPER_VID: u16 = 0x0483;
pub const FLIPPER_PID: u16 = 0x5740;
pub const FLIPPER_BAUD: u32 = 115200;
pub const FLIPPER_TIMEOUT: Duration = Duration::from_secs(3);

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
                serialport::SerialPortType::BluetoothPort => {
                    ("bluetooth".to_string(), None, None, None)
                }
                serialport::SerialPortType::PciPort => ("pci".to_string(), None, None, None),
                _ => ("unknown".to_string(), None, None, None),
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

/// Find a connected Flipper Zero by USB VID:PID.
pub fn find_flipper() -> Result<Option<PortInfo>, AppError> {
    let ports = serialport::available_ports()
        .map_err(|e| AppError::SerialError(format!("Cannot scan ports: {}", e)))?;

    for p in &ports {
        if let serialport::SerialPortType::UsbPort(usb) = &p.port_type
            && usb.vid == FLIPPER_VID
            && usb.pid == FLIPPER_PID
        {
            return Ok(Some(PortInfo {
                port_name: p.port_name.clone(),
                port_type: "usb".to_string(),
                manufacturer: usb.manufacturer.clone(),
                product: usb.product.clone(),
                serial_number: usb.serial_number.clone(),
            }));
        }
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// Connection state
// ---------------------------------------------------------------------------

pub type FlipperState = Arc<Mutex<Option<FlipperConnection>>>;

pub fn new_state() -> FlipperState {
    Arc::new(Mutex::new(None))
}

pub struct FlipperConnection {
    port_name: String,
}

impl FlipperConnection {
    fn open_port(&self) -> Result<Box<dyn serialport::SerialPort>, AppError> {
        serialport::new(&self.port_name, FLIPPER_BAUD)
            .timeout(FLIPPER_TIMEOUT)
            .open()
            .map_err(|e| AppError::SerialError(format!("Cannot open {}: {}", self.port_name, e)))
    }

    /// Send a CLI command line and read the response up to the next `>:` prompt.
    fn execute_command(&self, cmd: &str) -> Result<String, AppError> {
        let mut port = self.open_port()?;
        let cmd_line = format!("{}\r\n", cmd);
        port.write_all(cmd_line.as_bytes())
            .map_err(|e| AppError::SerialError(format!("Write error: {}", e)))?;
        port.flush()
            .map_err(|e| AppError::SerialError(format!("Flush error: {}", e)))?;

        let mut response = String::new();
        let mut buf = [0u8; 1024];
        let start = Instant::now();

        loop {
            if start.elapsed() > FLIPPER_TIMEOUT {
                return Err(AppError::SerialError("Timeout".to_string()));
            }
            match port.read(&mut buf) {
                Ok(0) => {
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
                Ok(n) => {
                    use std::str;
                    response.push_str(str::from_utf8(&buf[..n]).unwrap_or(""));
                    if response.contains(">:") {
                        break;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    return Err(AppError::SerialError("Timeout".to_string()));
                }
                Err(e) => return Err(AppError::SerialError(format!("Read error: {}", e))),
            }
        }

        if let Some(idx) = response.rfind(">:") {
            response.truncate(idx);
        }
        Ok(response.trim().to_string())
    }
}

// ---------------------------------------------------------------------------
// Connection management
// ---------------------------------------------------------------------------

pub fn connect(state: &FlipperState, port: &str) -> Result<bool, AppError> {
    if port.is_empty() {
        return Err(AppError::SerialError(
            "Port name cannot be empty".to_string(),
        ));
    }
    let _serial = serialport::new(port, FLIPPER_BAUD)
        .timeout(FLIPPER_TIMEOUT)
        .open()
        .map_err(|e| AppError::SerialError(format!("Cannot open {}: {}", port, e)))?;
    let mut guard = state
        .lock()
        .map_err(|_| AppError::SerialError("Mutex poisoned".to_string()))?;
    *guard = Some(FlipperConnection {
        port_name: port.to_string(),
    });
    Ok(true)
}

pub fn disconnect(state: &FlipperState) -> Result<bool, AppError> {
    let mut guard = state
        .lock()
        .map_err(|_| AppError::SerialError("Mutex poisoned".to_string()))?;
    *guard = None;
    Ok(true)
}

pub fn is_connected(state: &FlipperState) -> bool {
    state.lock().map(|g| g.is_some()).unwrap_or(false)
}

/// Scan for a Flipper Zero by VID:PID and connect to it.
pub fn autodetect_connect(state: &FlipperState) -> Result<bool, AppError> {
    match find_flipper()? {
        Some(port) => connect(state, &port.port_name),
        None => Err(AppError::SerialError(
            "Flipper Zero not found (VID:PID 0483:5740)".to_string(),
        )),
    }
}

// ---------------------------------------------------------------------------
// File operations (CLI-based, via Flipper's `storage` CLI commands)
// ---------------------------------------------------------------------------

fn with_connection<T>(
    state: &FlipperState,
    f: impl FnOnce(&FlipperConnection) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let guard = state
        .lock()
        .map_err(|_| AppError::SerialError("Mutex poisoned".into()))?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| AppError::SerialError("Not connected".into()))?;
    f(conn)
}

pub fn parse_list_output(path: &str, output: &str) -> Result<Vec<FileInfo>, AppError> {
    let mut entries = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
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

/// Parse the single-entry output of Flipper's `storage stat` CLI command,
/// e.g. `Size: 1234` for a file, or `Storage or directory` for a directory.
fn parse_stat_output(path: &str, output: &str) -> Result<FileInfo, AppError> {
    let name = path.rsplit('/').next().unwrap_or(path).to_string();
    let is_dir =
        output.to_lowercase().contains("directory") || output.to_lowercase().contains("storage");
    let size = output
        .lines()
        .find_map(|line| line.trim().strip_prefix("Size: "))
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    Ok(FileInfo {
        path: path.to_string(),
        name,
        size,
        is_dir,
        modified: None,
    })
}

pub fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let n = (b0 as u32) << 16 | (b1 as u32) << 8 | (b2 as u32);
        result.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

pub fn list_dir(state: &FlipperState, path: &str) -> Result<Vec<FileInfo>, AppError> {
    with_connection(state, |conn| {
        let output = conn.execute_command(&format!("storage list {}", path))?;
        parse_list_output(path, &output)
    })
}

pub fn read_file(state: &FlipperState, path: &str) -> Result<Vec<u8>, AppError> {
    with_connection(state, |conn| {
        let output = conn.execute_command(&format!("storage read {}", path))?;
        Ok(output.into_bytes())
    })
}

pub fn read_file_text(state: &FlipperState, path: &str) -> Result<String, AppError> {
    let bytes = read_file(state, path)?;
    String::from_utf8(bytes).map_err(|e| AppError::ParseError(format!("Not UTF-8: {}", e)))
}

pub fn write_file(state: &FlipperState, path: &str, data: &[u8]) -> Result<bool, AppError> {
    with_connection(state, |conn| {
        let encoded = base64_encode(data);
        conn.execute_command(&format!("storage write {} {}", path, encoded))?;
        Ok(true)
    })
}

pub fn write_file_text(state: &FlipperState, path: &str, content: &str) -> Result<bool, AppError> {
    write_file(state, path, content.as_bytes())
}

pub fn delete_path(state: &FlipperState, path: &str) -> Result<bool, AppError> {
    with_connection(state, |conn| {
        conn.execute_command(&format!("storage remove {}", path))?;
        Ok(true)
    })
}

pub fn mkdir_path(state: &FlipperState, path: &str) -> Result<bool, AppError> {
    with_connection(state, |conn| {
        conn.execute_command(&format!("storage mkdir {}", path))?;
        Ok(true)
    })
}

pub fn stat_path(state: &FlipperState, path: &str) -> Result<FileInfo, AppError> {
    with_connection(state, |conn| {
        let output = conn.execute_command(&format!("storage stat {}", path))?;
        parse_stat_output(path, &output)
    })
}

// ---------------------------------------------------------------------------
// Varint framing (for future Protobuf RPC)
// ---------------------------------------------------------------------------

pub fn encode_varint(buf: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        buf.push((value as u8) | 0x80);
        value >>= 7;
    }
    buf.push(value as u8);
}

pub fn read_varint<R: Read>(reader: &mut R, timeout: Duration) -> Result<u64, AppError> {
    let start = Instant::now();
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        if start.elapsed() > timeout {
            return Err(AppError::SerialError("Varint timeout".into()));
        }
        let mut byte = [0u8; 1];
        match reader.read_exact(&mut byte) {
            Ok(()) => {
                result |= (byte[0] as u64 & 0x7F) << shift;
                if byte[0] & 0x80 == 0 {
                    return Ok(result);
                }
                shift += 7;
                if shift >= 64 {
                    return Err(AppError::SerialError("Varint too long".into()));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                return Err(AppError::SerialError("Timeout".into()));
            }
            Err(e) => return Err(AppError::SerialError(format!("Read error: {}", e))),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_roundtrip() {
        let values = [0u64, 1, 127, 128, 255, 16383, 16384, 1000000];
        for &val in &values {
            let mut buf = Vec::new();
            encode_varint(&mut buf, val);
            let mut reader = &buf[..];
            let decoded = read_varint(&mut reader, Duration::from_secs(1)).unwrap();
            assert_eq!(val, decoded);
        }
    }

    #[test]
    fn test_varint_read_timeout_on_empty_input() {
        let mut reader: &[u8] = &[];
        let result = read_varint(&mut reader, Duration::from_millis(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"ab"), "YWI=");
        assert_eq!(base64_encode(b"abc"), "YWJj");
    }

    #[test]
    fn test_parse_list_output() {
        let output = "[D] subghz\n[F] test.sub 123\n[F] test2.sub 456";
        let result = parse_list_output("/ext", output).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result[0].is_dir);
        assert_eq!(result[0].name, "subghz");
    }

    #[test]
    fn test_parse_list_output_empty() {
        let result = parse_list_output("/ext", "").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_stat_output_file() {
        let info = parse_stat_output("/ext/test.sub", "Size: 42").unwrap();
        assert_eq!(info.name, "test.sub");
        assert_eq!(info.size, 42);
        assert!(!info.is_dir);
    }

    #[test]
    fn test_parse_stat_output_directory() {
        let info = parse_stat_output("/ext/subghz", "Storage or directory").unwrap();
        assert_eq!(info.name, "subghz");
        assert!(info.is_dir);
    }

    #[test]
    fn test_connect_rejects_empty_port_name() {
        let state = new_state();
        let result = connect(&state, "");
        assert!(matches!(result, Err(AppError::SerialError(_))));
    }

    #[test]
    fn test_is_connected_false_initially() {
        let state = new_state();
        assert!(!is_connected(&state));
    }

    #[test]
    fn test_disconnect_when_not_connected() {
        let state = new_state();
        assert!(disconnect(&state).is_ok());
        assert!(!is_connected(&state));
    }

    #[test]
    fn test_list_dir_not_connected() {
        let state = new_state();
        let result = list_dir(&state, "/ext");
        assert!(matches!(result, Err(AppError::SerialError(_))));
    }

    #[test]
    fn test_stat_path_not_connected() {
        let state = new_state();
        let result = stat_path(&state, "/ext/test.sub");
        assert!(matches!(result, Err(AppError::SerialError(_))));
    }

    #[test]
    fn test_delete_path_not_connected() {
        let state = new_state();
        let result = delete_path(&state, "/ext/test.sub");
        assert!(matches!(result, Err(AppError::SerialError(_))));
    }

    #[test]
    fn test_mkdir_path_not_connected() {
        let state = new_state();
        let result = mkdir_path(&state, "/ext/newdir");
        assert!(matches!(result, Err(AppError::SerialError(_))));
    }
}
