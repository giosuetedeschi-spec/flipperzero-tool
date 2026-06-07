use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct PortInfo {
    pub name: String,
    pub port_type: String,
    pub description: Option<String>,
}

static mut CONNECTED: bool = false;

pub fn list_ports() -> Result<Vec<PortInfo>, String> {
    serialport::available_ports()
        .map(|ports| {
            ports.into_iter().map(|p| PortInfo {
                name: p.port_name.clone(),
                port_type: format!("{:?}", p.port_type),
                description: match p.port_type {
                    serialport::SerialPortType::UsbPort(info) => {
                        Some(format!("{:04x}:{:04x}", info.vid, info.pid))
                    }
                    _ => None,
                },
            }).collect()
        })
        .map_err(|e| format!("Serial error: {}", e))
}

pub fn connect(port: &str) -> Result<bool, String> {
    // TODO: implement actual connection via Protobuf
    unsafe { CONNECTED = true; }
    Ok(true)
}

pub fn disconnect() -> Result<bool, String> {
    unsafe { CONNECTED = false; }
    Ok(true)
}

pub fn read_file(path: &str) -> Result<Vec<u8>, String> {
    // TODO: implement via serial Protobuf
    Err(format!("Not implemented yet: read {}", path))
}

pub fn write_file(path: &str, _data: &[u8]) -> Result<bool, String> {
    // TODO: implement via serial Protobuf
    Err(format!("Not implemented yet: write {}", path))
}

pub fn list_dir(path: &str) -> Result<Vec<super::commands::FileInfo>, String> {
    // TODO: implement via serial Protobuf
    Err(format!("Not implemented yet: list {}", path))
}
