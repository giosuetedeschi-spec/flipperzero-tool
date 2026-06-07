use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedFile {
    pub file_type: String,
    pub fields: Vec<serde_json::Value>,
    pub raw_preview: String,
}

// Serial commands
#[tauri::command]
pub fn serial_list_ports() -> Result<Vec<serial::PortInfo>, String> {
    serial::list_ports()
}

#[tauri::command]
pub fn serial_connect(port: String) -> Result<bool, String> {
    serial::connect(&port)
}

#[tauri::command]
pub fn serial_disconnect() -> Result<bool, String> {
    serial::disconnect()
}

#[tauri::command]
pub fn serial_read_file(path: String) -> Result<Vec<u8>, String> {
    serial::read_file(&path)
}

#[tauri::command]
pub fn serial_write_file(path: String, data: Vec<u8>) -> Result<bool, String> {
    serial::write_file(&path, &data)
}

#[tauri::command]
pub fn serial_list_dir(path: String) -> Result<Vec<FileInfo>, String> {
    serial::list_dir(&path)
}

// VFS commands
#[tauri::command]
pub fn fs_index_device() -> Result<bool, String> {
    vfs::reindex()
}

#[tauri::command]
pub fn fs_get_cached_tree() -> Result<Vec<FileInfo>, String> {
    vfs::get_tree()
}

// Parser commands
#[tauri::command]
pub fn parser_parse_sub(data: String) -> Result<ParsedFile, String> {
    parsers::parse_sub(&data)
}

#[tauri::command]
pub fn parser_parse_ir(data: String) -> Result<ParsedFile, String> {
    parsers::parse_ir(&data)
}

#[tauri::command]
pub fn parser_parse_nfc(data: String) -> Result<ParsedFile, String> {
    parsers::parse_nfc(&data)
}

// uFBT commands
#[tauri::command]
pub fn ufbt_new_project(name: String, path: String) -> Result<String, String> {
    std::process::Command::new("ufbt")
        .args(["create", "app", "--name", &name])
        .current_dir(&path)
        .output()
        .map_err(|e| format!("ufbt not found: {}", e))?;
    Ok(format!("Created plugin: {}", name))
}

#[tauri::command]
pub fn ufbt_compile() -> Result<String, String> {
    let output = std::process::Command::new("ufbt")
        .arg("build")
        .output()
        .map_err(|e| format!("ufbt build failed: {}", e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub fn ufbt_launch() -> Result<String, String> {
    let output = std::process::Command::new("ufbt")
        .arg("launch")
        .output()
        .map_err(|e| format!("ufbt launch failed: {}", e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
