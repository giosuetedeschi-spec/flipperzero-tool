use rusqlite::Connection;
use tauri::{AppHandle, Manager};
use std::path::PathBuf;
use super::errors::AppError;
use super::commands::FileInfo;

fn db_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_data_dir()
        .map_err(|e| AppError::DbError(format!("Cannot get app data dir: {}", e)))?;
    std::fs::create_dir_all(&dir).map_err(AppError::from)?;
    Ok(dir.join("flipper_cache.db"))
}

pub fn init_cache(app: &AppHandle) -> Result<(), AppError> {
    let path = db_path(app)?;

    let conn = Connection::open(&path).map_err(AppError::from)?;

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
    ).map_err(AppError::from)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_cache (
            path TEXT PRIMARY KEY,
            content BLOB,
            cached_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).map_err(AppError::from)?;

    Ok(())
}

pub fn reindex() -> Result<bool, AppError> {
    // TODO: connect to Flipper via serial, traverse filesystem, populate DB
    Ok(true)
}

pub fn get_tree() -> Result<Vec<FileInfo>, AppError> {
    // TODO: read from SQLite cache
    Ok(vec![])
}
