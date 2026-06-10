use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use super::errors::AppError;
use super::parsers::ParsedFile;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<String>,
}

// ---------------------------------------------------------------------------
// Local filesystem operations (mock SD card for offline development)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<FileInfo>, AppError> {
    let dir_path = Path::new(&path);

    if !dir_path.exists() {
        return Err(AppError::NotFound(format!("Directory not found: {}", path)));
    }

    if !dir_path.is_dir() {
        return Err(AppError::General(format!("Path is not a directory: {}", path)));
    }

    let mut entries = Vec::new();

    for entry in fs::read_dir(dir_path).map_err(|e| match e.kind() {
        std::io::ErrorKind::PermissionDenied => AppError::PermissionDenied(format!("Cannot read directory: {}", path)),
        _ => AppError::from(e),
    })? {
        let entry = entry.map_err(AppError::from)?;
        let metadata = entry.metadata().map_err(AppError::from)?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_path = entry.path().to_string_lossy().to_string();

        let modified = metadata.modified()
            .ok()
            .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs().to_string());

        entries.push(FileInfo {
            path: file_path,
            name: file_name,
            size: metadata.len(),
            is_dir: metadata.is_dir(),
            modified,
        });
    }

    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
    Ok(entries)
}

#[tauri::command]
pub fn move_file(source: String, dest: String) -> Result<(), AppError> {
    let src = Path::new(&source);
    let dst = Path::new(&dest);

    if !src.exists() {
        return Err(AppError::NotFound(format!("Source not found: {}", source)));
    }

    if dst.exists() {
        return Err(AppError::AlreadyExists(format!("Destination already exists: {}", dest)));
    }

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => AppError::PermissionDenied(format!("Cannot create parent directory: {}", parent.display())),
            _ => AppError::from(e),
        })?;
    }

    fs::rename(src, dst).map_err(|e| match e.kind() {
        std::io::ErrorKind::PermissionDenied => AppError::PermissionDenied(format!("Cannot move: {}", e)),
        _ => AppError::from(e),
    })?;

    Ok(())
}

#[tauri::command]
pub fn find_files(path: String, pattern: String) -> Result<Vec<FileInfo>, AppError> {
    let dir_path = Path::new(&path);

    if !dir_path.exists() {
        return Err(AppError::NotFound(format!("Path not found: {}", path)));
    }

    let pattern_lower = pattern.to_lowercase();
    let mut results = Vec::new();

    fn search_dir(
        dir: &Path,
        pattern: &str,
        results: &mut Vec<FileInfo>,
    ) -> Result<(), AppError> {
        for entry in fs::read_dir(dir).map_err(AppError::from)? {
            let entry = entry.map_err(AppError::from)?;
            let name = entry.file_name().to_string_lossy().to_string();

            if name.to_lowercase().contains(pattern) {
                let metadata = entry.metadata().map_err(AppError::from)?;
                let modified = metadata.modified()
                    .ok()
                    .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs().to_string());

                results.push(FileInfo {
                    path: entry.path().to_string_lossy().to_string(),
                    name,
                    size: metadata.len(),
                    is_dir: metadata.is_dir(),
                    modified,
                });
            }

            if entry.file_type().map_err(AppError::from)?.is_dir() {
                search_dir(&entry.path(), pattern, results)?;
            }
        }
        Ok(())
    }

    search_dir(dir_path, &pattern_lower, &mut results)?;
    Ok(results)
}

#[tauri::command]
pub fn create_file_from_template(path: String, ext: String) -> Result<String, AppError> {
    let file_path = PathBuf::from(format!("{}.{}", path, ext));

    if file_path.exists() {
        return Err(AppError::AlreadyExists(format!(
            "File already exists: {}",
            file_path.display()
        )));
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => {
                AppError::PermissionDenied(format!("Cannot create directory: {}", parent.display()))
            }
            _ => AppError::from(e),
        })?;
    }

    fs::write(&file_path, "").map_err(|e| match e.kind() {
        std::io::ErrorKind::PermissionDenied => {
            AppError::PermissionDenied(format!("Cannot create file: {}", file_path.display()))
        }
        _ => AppError::from(e),
    })?;

    Ok(file_path.to_string_lossy().to_string())
}

// ---------------------------------------------------------------------------
// Serial commands (async via tokio — no UI blocking)
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn serial_list_ports() -> Result<Vec<super::serial::PortInfo>, AppError> {
    super::serial::list_ports()
}

#[tauri::command]
pub async fn serial_connect(
    state: tauri::State<'_, super::serial::FlipperState>,
    port: String,
) -> Result<bool, AppError> {
    super::serial::connect(&state, &port)
}

#[tauri::command]
pub async fn serial_disconnect(
    state: tauri::State<'_, super::serial::FlipperState>,
) -> Result<bool, AppError> {
    super::serial::disconnect(&state)
}

#[tauri::command]
pub async fn serial_read_file(
    state: tauri::State<'_, super::serial::FlipperState>,
    path: String,
) -> Result<String, AppError> {
    super::serial::read_file_text(&state, &path)
}

#[tauri::command]
pub async fn serial_write_file(
    state: tauri::State<'_, super::serial::FlipperState>,
    path: String,
    data: String,
) -> Result<bool, AppError> {
    super::serial::write_file_text(&state, &path, &data)
}

#[tauri::command]
pub async fn serial_list_dir(
    state: tauri::State<'_, super::serial::FlipperState>,
    path: String,
) -> Result<Vec<FileInfo>, AppError> {
    super::serial::list_dir(&state, &path)
}

#[tauri::command]
pub fn serial_is_connected(
    state: tauri::State<'_, super::serial::FlipperState>,
) -> bool {
    super::serial::is_connected(&state)
}

#[tauri::command]
pub async fn local_read_file(path: String) -> Result<String, AppError> {
    let bytes = fs::read(&path)
        .map_err(|e| AppError::from(e))?;
    String::from_utf8(bytes)
        .map_err(|e| AppError::ParseError(format!("File is not valid UTF-8: {}", e)))
}

#[tauri::command]
pub async fn local_write_file(path: String, data: String) -> Result<bool, AppError> {
    // Safe save: write to temp file first, then rename
    let path_obj = Path::new(&path);
    let temp_path = path_obj.with_extension("tmp");

    fs::write(&temp_path, data.as_bytes())
        .map_err(|e| AppError::from(e))?;

    fs::rename(&temp_path, path_obj)
        .map_err(|e| AppError::from(e))?;

    Ok(true)
}

// ---------------------------------------------------------------------------
// VFS commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn fs_index_device() -> Result<bool, AppError> {
    super::vfs::reindex()
}

#[tauri::command]
pub fn fs_get_cached_tree() -> Result<Vec<FileInfo>, AppError> {
    super::vfs::get_tree()
}

// ---------------------------------------------------------------------------
// Parser commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn parser_parse_sub(data: String) -> Result<ParsedFile, AppError> {
    super::parsers::parse_sub(&data)
}

#[tauri::command]
pub fn parser_parse_ir(data: String) -> Result<ParsedFile, AppError> {
    super::parsers::parse_ir(&data)
}

#[tauri::command]
pub fn parser_parse_nfc(data: String) -> Result<ParsedFile, AppError> {
    super::parsers::parse_nfc(&data)
}

#[tauri::command]
pub fn open_in_system(path: String) -> Result<(), AppError> {
    open::that(&path).map_err(|e| AppError::General(format!("Cannot open: {}", e)))
}

// ---------------------------------------------------------------------------
// uFBT commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn ufbt_new_project(name: String, path: String) -> Result<String, AppError> {
    let output = std::process::Command::new("ufbt")
        .args(["create", "app", "--name", &name])
        .current_dir(&path)
        .output()
        .map_err(|e| AppError::General(format!("ufbt not found: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::General(format!("ufbt failed: {}", stderr)));
    }

    Ok(format!("Created plugin: {}", name))
}

#[tauri::command]
pub fn ufbt_compile() -> Result<String, AppError> {
    let output = std::process::Command::new("ufbt")
        .arg("build")
        .output()
        .map_err(|e| AppError::General(format!("ufbt build failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::General(format!("ufbt build failed: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ---------------------------------------------------------------------------
// Additional CRUD commands (FASE 1)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn rename_file(path: String, new_name: String) -> Result<String, AppError> {
    let src = Path::new(&path);
    if !src.exists() {
        return Err(AppError::NotFound(format!("File not found: {}", path)));
    }
    let parent = src.parent().ok_or_else(|| {
        AppError::General("Cannot determine parent directory".to_string())
    })?;
    let new_path = parent.join(new_name);
    if new_path.exists() {
        return Err(AppError::AlreadyExists(format!(
            "File already exists: {}",
            new_path.display()
        )));
    }
    fs::rename(src, &new_path).map_err(AppError::from)?;
    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn delete_file(path: String) -> Result<(), AppError> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound(format!("File not found: {}", path)));
    }
    if p.is_dir() {
        fs::remove_dir_all(p).map_err(AppError::from)?;
    } else {
        fs::remove_file(p).map_err(AppError::from)?;
    }
    Ok(())
}

#[tauri::command]
pub fn copy_file(source: String, dest: String) -> Result<(), AppError> {
    let src = Path::new(&source);
    if !src.exists() {
        return Err(AppError::NotFound(format!("Source not found: {}", source)));
    }
    let dst = Path::new(&dest);
    if dst.exists() {
        return Err(AppError::AlreadyExists(format!("Destination exists: {}", dest)));
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(AppError::from)?;
    }
    if src.is_dir() {
        // Simple recursive copy
        copy_dir_all(src, dst).map_err(AppError::from)?;
    } else {
        fs::copy(src, dst).map_err(AppError::from)?;
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(&entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_file_content(path: String) -> Result<String, AppError> {
    let bytes = fs::read(&path).map_err(AppError::from)?;
    String::from_utf8(bytes)
        .map_err(|e| AppError::ParseError(format!("Not valid UTF-8: {}", e)))
}

#[tauri::command]
pub fn write_file_content(path: String, content: String) -> Result<(), AppError> {
    let path_obj = Path::new(&path);
    // Safe save: write to temp then rename
    let temp = path_obj.with_extension("tmp");
    fs::write(&temp, content.as_bytes()).map_err(AppError::from)?;
    fs::rename(&temp, path_obj).map_err(AppError::from)?;
    Ok(())
}

#[tauri::command]
pub fn get_app_paths() -> Result<serde_json::Value, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::General("Cannot find home directory".into()))?;
    let desktop = dirs::desktop_dir()
        .ok_or_else(|| AppError::General("No desktop directory".into()))?;
    let documents = dirs::document_dir()
        .ok_or_else(|| AppError::General("No documents directory".into()))?;
    Ok(serde_json::json!({
        "home": home.to_string_lossy(),
        "desktop": desktop.to_string_lossy(),
        "documents": documents.to_string_lossy(),
    }))
}

// ---------------------------------------------------------------------------
// uFBT commands (FASE 4)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn ufbt_is_installed() -> bool {
    super::ufbt::is_ufbt_installed()
}

#[tauri::command]
pub fn ufbt_get_version() -> Result<String, AppError> {
    super::ufbt::get_ufbt_version()
}

#[tauri::command]
pub fn ufbt_get_sdk_version() -> Result<String, AppError> {
    super::ufbt::get_sdk_version()
}

#[tauri::command]
pub fn ufbt_install() -> Result<String, AppError> {
    super::ufbt::ufbt_install()
}

#[tauri::command]
pub fn ufbt_update() -> Result<String, AppError> {
    super::ufbt::ufbt_update()
}

#[tauri::command]
pub fn ufbt_create(name: String, path: String) -> Result<String, AppError> {
    super::ufbt::create_fap_project(&name, &path)
}

#[tauri::command]
pub fn ufbt_build(path: String) -> Result<String, AppError> {
    super::ufbt::build_fap(&path)
}

#[tauri::command]
pub fn ufbt_deploy(path: String) -> Result<String, AppError> {
    super::ufbt::deploy_fap(&path)
}

#[tauri::command]
pub fn ufbt_clean(path: String) -> Result<String, AppError> {
    super::ufbt::clean_fap(&path)
}
