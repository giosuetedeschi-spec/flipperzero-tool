use rusqlite::{Connection, Result as SqlResult};
use tauri::AppHandle;
use std::path::PathBuf;

fn db_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap_or_default().join("flipper_cache.db")
}

pub fn init_cache(app: &AppHandle) -> Result<(), String> {
    let path = db_path(app);
    std::fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| format!("Dir create failed: {}", e))?;
    
    let conn = Connection::open(&path)
        .map_err(|e| format!("DB open failed: {}", e))?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            path TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            size INTEGER DEFAULT 0,
            is_dir INTEGER DEFAULT 0,
            modified TEXT,
            parent TEXT
        )",
        [],
    ).map_err(|e| format!("Table create failed: {}", e))?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_cache (
            path TEXT PRIMARY KEY,
            content BLOB,
            cached_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).map_err(|e| format!("Cache table create failed: {}", e))?;
    
    Ok(())
}

pub fn reindex() -> Result<bool, String> {
    // TODO: connect to Flipper, traverse filesystem, populate DB
    Ok(true)
}

pub fn get_tree() -> Result<Vec<super::commands::FileInfo>, String> {
    // TODO: read from SQLite cache
    Ok(vec![])
}
