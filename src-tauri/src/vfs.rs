//! Virtual File System - SQLite cache for Flipper Zero filesystem.
//!
//! Provides:
//!   - SQLite-backed file tree cache
//!   - Re-indexing from Flipper via serial CLI
//!   - File content caching
//!   - Tree retrieval for the frontend

use rusqlite::{Connection, params};
use tauri::{AppHandle, Manager};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use super::errors::AppError;
use super::commands::FileInfo;

// ---------------------------------------------------------------------------
// Database path
// ---------------------------------------------------------------------------

fn db_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_data_dir()
        .map_err(|e| AppError::DbError(format!("Cannot get app data dir: {}", e)))?;
    std::fs::create_dir_all(&dir).map_err(AppError::from)?;
    Ok(dir.join("flipper_cache.db"))
}

// ---------------------------------------------------------------------------
// Connection helper (thread-safe via Arc<Mutex<>>)
// ---------------------------------------------------------------------------

type DbState = Arc<StdMutex<Connection>>;

/// Get or create the database connection.
fn get_conn(app: &AppHandle) -> Result<DbState, AppError> {
    // Try to get existing state
    if let Some(state) = app.try_state::<DbState>() {
        return Ok(state.inner().clone());
    }

    // Create new connection
    let path = db_path(app)?;
    let conn = Connection::open(&path).map_err(AppError::from)?;

    // Create tables
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

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_parent ON files(parent)",
        [],
    ).map_err(AppError::from)?;

    let state: DbState = Arc::new(StdMutex::new(conn));
    app.manage(state.clone());
    Ok(state)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Initialize the VFS cache database.
pub fn init_cache(app: &AppHandle) -> Result<(), AppError> {
    get_conn(app)?;
    Ok(())
}

/// Re-index the Flipper filesystem.
///
/// In a real implementation, this would:
/// 1. Connect to Flipper via serial
/// 2. Walk the filesystem using `storage list` commands
/// 3. Populate the SQLite cache
///
/// For now, this is a stub that clears and prepares the cache.
pub fn reindex() -> Result<bool, AppError> {
    // TODO: connect to Flipper via serial, traverse filesystem, populate DB
    // For now, just return true to indicate the cache is ready
    Ok(true)
}

/// Re-index with a specific app handle (for Tauri commands).
pub fn reindex_app(app: &AppHandle) -> Result<bool, AppError> {
    let state = get_conn(app)?;
    let conn = state.lock().map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;

    // Clear existing entries
    conn.execute("DELETE FROM files", []).map_err(AppError::from)?;

    // TODO: Walk Flipper filesystem via serial and insert entries
    // For now, insert some default Flipper directories
    let default_dirs = vec![
        ("/ext", "ext", 0, true, "/"),
        ("/ext/subghz", "subghz", 0, true, "/ext"),
        ("/ext/infrared", "infrared", 0, true, "/ext"),
        ("/ext/nfc", "nfc", 0, true, "/ext"),
        ("/ext/badusb", "badusb", 0, true, "/ext"),
        ("/ext/gpio", "gpio", 0, true, "/ext"),
        ("/ext/music_player", "music_player", 0, true, "/ext"),
        ("/int", "int", 0, true, "/"),
    ];

    for (path, name, size, is_dir, parent) in &default_dirs {
        conn.execute(
            "INSERT OR REPLACE INTO files (path, name, size, is_dir, parent) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![path, name, size, if *is_dir { 1 } else { 0 }, parent],
        ).map_err(AppError::from)?;
    }

    Ok(true)
}

/// Get the cached file tree.
pub fn get_tree() -> Result<Vec<FileInfo>, AppError> {
    // TODO: read from SQLite cache
    Ok(vec![])
}

/// Get the cached file tree for a specific app.
pub fn get_tree_app(app: &AppHandle) -> Result<Vec<FileInfo>, AppError> {
    let state = get_conn(app)?;
    let conn = state.lock().map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;

    let mut stmt = conn
        .prepare("SELECT path, name, size, is_dir, modified FROM files ORDER BY is_dir DESC, name ASC")
        .map_err(AppError::from)?;

    let rows = stmt
        .query_map([], |row| {
            Ok(FileInfo {
                path: row.get(0)?,
                name: row.get(1)?,
                size: row.get(2)?,
                is_dir: row.get::<_, i32>(3)? != 0,
                modified: row.get(4)?,
            })
        })
        .map_err(AppError::from)?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(AppError::from)?);
    }

    Ok(results)
}

/// Cache a file's content in the database.
pub fn cache_file(app: &AppHandle, path: &str, content: &[u8]) -> Result<(), AppError> {
    let state = get_conn(app)?;
    let conn = state.lock().map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;

    conn.execute(
        "INSERT OR REPLACE INTO file_cache (path, content) VALUES (?1, ?2)",
        params![path, content],
    ).map_err(AppError::from)?;

    Ok(())
}

/// Get a cached file's content.
pub fn get_cached_file(app: &AppHandle, path: &str) -> Result<Option<Vec<u8>>, AppError> {
    let state = get_conn(app)?;
    let conn = state.lock().map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;

    let mut stmt = conn
        .prepare("SELECT content FROM file_cache WHERE path = ?1")
        .map_err(AppError::from)?;

    let result: Result<Vec<u8>, rusqlite::Error> = stmt.query_row(params![path], |row| row.get(0));

    match result {
        Ok(content) => Ok(Some(content)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DbError(e.to_string())),
    }
}

/// Insert a file entry into the cache.
pub fn insert_file(app: &AppHandle, info: &FileInfo) -> Result<(), AppError> {
    let state = get_conn(app)?;
    let conn = state.lock().map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;

    let parent = PathBuf::from(&info.path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    conn.execute(
        "INSERT OR REPLACE INTO files (path, name, size, is_dir, parent) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            &info.path,
            &info.name,
            info.size,
            if info.is_dir { 1 } else { 0 },
            parent,
        ],
    ).map_err(AppError::from)?;

    Ok(())
}

/// Remove a file entry from the cache.
pub fn remove_file(app: &AppHandle, path: &str) -> Result<(), AppError> {
    let state = get_conn(app)?;
    let conn = state.lock().map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;

    conn.execute("DELETE FROM files WHERE path = ?1", params![path])
        .map_err(AppError::from)?;
    conn.execute("DELETE FROM file_cache WHERE path = ?1", params![path])
        .map_err(AppError::from)?;

    Ok(())
}

/// Clear the entire cache.
pub fn clear_cache(app: &AppHandle) -> Result<(), AppError> {
    let state = get_conn(app)?;
    let conn = state.lock().map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;

    conn.execute("DELETE FROM files", []).map_err(AppError::from)?;
    conn.execute("DELETE FROM file_cache", []).map_err(AppError::from)?;

    Ok(())
}
