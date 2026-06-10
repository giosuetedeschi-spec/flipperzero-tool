use serde::Serialize;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use super::errors::AppError;

#[derive(Serialize, Debug, Clone)]
pub struct PortInfo {
    pub name: String,
    pub port_type: String,
    pub description: Option<String>,
}

/// Timeout for all serial operations (3 seconds as per requirements)
const SERIAL_TIMEOUT: Duration = Duration::from_secs(3);
const BAUD_RATE: u32 = 115200;

/// Flipper CLI prompt marker
const FLIPPER_PROMPT: &str = ">:";

static CONNECTED: AtomicBool = AtomicBool::new(false);
static PORT_NAME: Mutex<String> = Mutex::new(String::new());

// ---------------------------------------------------------------------------
// Port discovery
// ---------------------------------------------------------------------------

pub fn list_ports() -> Result<Vec<PortInfo>, AppError> {
    let ports = serialport::available_ports()
        .map_err(|e| AppError::SerialError(e.to_string()))?;

    Ok(ports.into_iter().map(|p| PortInfo {
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

// ---------------------------------------------------------------------------
// Connection management
// ---------------------------------------------------------------------------

pub fn connect(port: &str) -> Result<bool, AppError> {
    if port.is_empty() {
        return Err(AppError::SerialError("Port name cannot be empty".to_string()));
    }

    // Try to open the port to verify it exists and is accessible
    let _serial = serialport::new(port, BAUD_RATE)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .map_err(|e| AppError::SerialError(format!("Cannot open port {}: {}", port, e)))?;

    let mut name = PORT_NAME.lock().map_err(|_| {
        AppError::SerialError("Mutex poisoned".to_string())
    })?;
    *name = port.to_string();
    CONNECTED.store(true, Ordering::SeqCst);

    Ok(true)
}

pub fn disconnect() -> Result<bool, AppError> {
    CONNECTED.store(false, Ordering::SeqCst);
    let mut name = PORT_NAME.lock().map_err(|_| {
        AppError::SerialError("Mutex poisoned".to_string())
    })?;
    *name = String::new();
    Ok(true)
}

pub fn is_connected() -> bool {
    CONNECTED.load(Ordering::SeqCst)
}

fn get_port_name() -> Result<String, AppError> {
    let name = PORT_NAME.lock().map_err(|_| {
        AppError::SerialError("Mutex poisoned".to_string())
    })?;
    if name.is_empty() {
        return Err(AppError::SerialError("Not connected to any port".to_string()));
    }
    Ok(name.clone())
}

// ---------------------------------------------------------------------------
// Low-level serial command execution
// ---------------------------------------------------------------------------

fn execute_flipper_command(command: &str) -> Result<String, AppError> {
    let port_name = get_port_name()?;

    let mut serial = serialport::new(&port_name, BAUD_RATE)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .map_err(|e| AppError::SerialError(format!("Cannot open serial port: {}", e)))?;

    // Clear any pending input
    let _ = serial.flush();

    // Send the command as a text line
    let cmd_line = format!("{}\r\n", command);
    serial.write_all(cmd_line.as_bytes())
        .map_err(|e| AppError::SerialError(format!("Write error: {}", e)))?;

    // Read response with timeout handling
    let mut response = String::new();
    let mut buf = [0u8; 1024];
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > SERIAL_TIMEOUT {
            return Err(AppError::SerialError(
                "Timeout: Flipper did not respond within 3 seconds".to_string()
            ));
        }

        match serial.read(&mut buf) {
            Ok(0) => {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                response.push_str(&chunk);

                if response.contains(FLIPPER_PROMPT) {
                    break;
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    return Err(AppError::SerialError(
                        "Timeout: Flipper did not respond within 3 seconds".to_string()
                    ));
                }
                return Err(AppError::SerialError(format!("Read error: {}", e)));
            }
        }
    }

    // Strip the prompt from the response
    if let Some(idx) = response.rfind(FLIPPER_PROMPT) {
        response.truncate(idx);
    }

    // Strip the echo of the command itself (first line)
    let lines: Vec<&str> = response.lines().collect();
    if !lines.is_empty() && lines[0].trim() == command.trim() {
        Ok(lines[1..].join("\n").trim().to_string())
    } else {
        Ok(response.trim().to_string())
    }
}

// ---------------------------------------------------------------------------
// File operations via Flipper CLI
// ---------------------------------------------------------------------------

pub fn read_file(path: &str) -> Result<Vec<u8>, AppError> {
    if path.is_empty() {
        return Err(AppError::General("Path cannot be empty".to_string()));
    }

    if !is_connected() {
        return Err(AppError::SerialError("Not connected to Flipper".to_string()));
    }

    let cmd = format!("storage read {}", path);
    let response = execute_flipper_command(&cmd)?;
    Ok(response.into_bytes())
}

pub fn read_file_text(path: &str) -> Result<String, AppError> {
    let bytes = read_file(path)?;
    String::from_utf8(bytes)
        .map_err(|e| AppError::ParseError(format!("File is not valid UTF-8: {}", e)))
}

/// Safe save: verify connection before writing
pub fn write_file(path: &str, data: &[u8]) -> Result<bool, AppError> {
    if path.is_empty() {
        return Err(AppError::General("Path cannot be empty".to_string()));
    }

    if !is_connected() {
        return Err(AppError::SerialError("Not connected to Flipper".to_string()));
    }

    // Safe save: verify connection is alive first
    let _ping = execute_flipper_command("device_info")?;

    // Use base64 to avoid escaping issues
    let encoded = base64_encode(data);
    let cmd = format!("storage write {} {}", path, encoded);
    let _response = execute_flipper_command(&cmd)?;

    Ok(true)
}

pub fn write_file_text(path: &str, content: &str) -> Result<bool, AppError> {
    write_file(path, content.as_bytes())
}

pub fn list_dir(path: &str) -> Result<Vec<super::commands::FileInfo>, AppError> {
    if path.is_empty() {
        return Err(AppError::General("Path cannot be empty".to_string()));
    }

    if !is_connected() {
        return Err(AppError::SerialError("Not connected to Flipper".to_string()));
    }

    let cmd = format!("storage list {}", path);
    let response = execute_flipper_command(&cmd)?;

    let mut entries = Vec::new();
    for line in response.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("[F] ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if !parts.is_empty() {
                let name = parts[0].to_string();
                let size = parts.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                entries.push(super::commands::FileInfo {
                    path: format!("{}/{}", path.trim_end_matches('/'), name),
                    name,
                    size,
                    is_dir: false,
                    modified: None,
                });
            }
        } else if let Some(rest) = line.strip_prefix("[D] ") {
            let name = rest.trim().to_string();
            entries.push(super::commands::FileInfo {
                path: format!("{}/{}", path.trim_end_matches('/'), name),
                name,
                size: 0,
                is_dir: true,
                modified: None,
            });
        }
    }

    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
    Ok(entries)
}

// ---------------------------------------------------------------------------
// Base64 helper
// ---------------------------------------------------------------------------

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
