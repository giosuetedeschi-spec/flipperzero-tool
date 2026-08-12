//! Virtual File System - SQLite cache for Flipper Zero filesystem.
//!
//! Provides:
//!   - SQLite-backed file tree cache
//!   - Re-indexing from Flipper via serial CLI
//!   - File content caching
//!   - Tree retrieval for the frontend
//!
//! The `*_conn` functions below hold all the actual SQL logic and take a
//! plain `&Connection`, so they can be unit-tested against an in-memory
//! database without a running Tauri app. The `*_app` functions are thin
//! wrappers that resolve the app's managed connection and delegate.

use super::commands::FileInfo;
use super::errors::AppError;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tauri::{AppHandle, Manager};

// ---------------------------------------------------------------------------
// Database path
// ---------------------------------------------------------------------------

fn db_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::DbError(format!("Cannot get app data dir: {}", e)))?;
    std::fs::create_dir_all(&dir).map_err(AppError::from)?;
    Ok(dir.join("flipper_cache.db"))
}

// ---------------------------------------------------------------------------
// Schema
// ---------------------------------------------------------------------------

fn init_schema(conn: &Connection) -> Result<(), AppError> {
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
    )
    .map_err(AppError::from)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_cache (
            path TEXT PRIMARY KEY,
            content BLOB,
            cached_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .map_err(AppError::from)?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_parent ON files(parent)",
        [],
    )
    .map_err(AppError::from)?;

    // The Signal Radar tables live in the same database file rather than a
    // second one, so a single connection serves both the file cache and the
    // signal history.
    crate::signals::store::init_schema(conn)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Connection helper (thread-safe via Arc<Mutex<>>)
// ---------------------------------------------------------------------------

pub(crate) type DbState = Arc<StdMutex<Connection>>;

/// Get or create the database connection.
pub(crate) fn get_conn(app: &AppHandle) -> Result<DbState, AppError> {
    // Try to get existing state
    if let Some(state) = app.try_state::<DbState>() {
        return Ok(state.inner().clone());
    }

    // Create new connection
    let path = db_path(app)?;
    let conn = Connection::open(&path).map_err(AppError::from)?;
    init_schema(&conn)?;

    let state: DbState = Arc::new(StdMutex::new(conn));
    app.manage(state.clone());
    Ok(state)
}

// ---------------------------------------------------------------------------
// Connection-based core logic (unit-testable)
// ---------------------------------------------------------------------------

fn reindex_conn(conn: &Connection) -> Result<bool, AppError> {
    conn.execute("DELETE FROM files", [])
        .map_err(AppError::from)?;

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

fn get_tree_conn(conn: &Connection) -> Result<Vec<FileInfo>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT path, name, size, is_dir, modified FROM files ORDER BY is_dir DESC, name ASC",
        )
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

fn cache_file_conn(conn: &Connection, path: &str, content: &[u8]) -> Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO file_cache (path, content) VALUES (?1, ?2)",
        params![path, content],
    )
    .map_err(AppError::from)?;

    Ok(())
}

fn get_cached_file_conn(conn: &Connection, path: &str) -> Result<Option<Vec<u8>>, AppError> {
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

fn insert_file_conn(conn: &Connection, info: &FileInfo) -> Result<(), AppError> {
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

fn remove_file_conn(conn: &Connection, path: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM files WHERE path = ?1", params![path])
        .map_err(AppError::from)?;
    conn.execute("DELETE FROM file_cache WHERE path = ?1", params![path])
        .map_err(AppError::from)?;

    Ok(())
}

fn clear_cache_conn(conn: &Connection) -> Result<(), AppError> {
    conn.execute("DELETE FROM files", [])
        .map_err(AppError::from)?;
    conn.execute("DELETE FROM file_cache", [])
        .map_err(AppError::from)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Public API (Tauri app-handle wrappers)
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
    let conn = state
        .lock()
        .map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;
    reindex_conn(&conn)
}

/// Get the cached file tree.
pub fn get_tree() -> Result<Vec<FileInfo>, AppError> {
    // TODO: read from SQLite cache
    Ok(vec![])
}

/// Get the cached file tree for a specific app.
pub fn get_tree_app(app: &AppHandle) -> Result<Vec<FileInfo>, AppError> {
    let state = get_conn(app)?;
    let conn = state
        .lock()
        .map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;
    get_tree_conn(&conn)
}

/// Cache a file's content in the database.
pub fn cache_file(app: &AppHandle, path: &str, content: &[u8]) -> Result<(), AppError> {
    let state = get_conn(app)?;
    let conn = state
        .lock()
        .map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;
    cache_file_conn(&conn, path, content)
}

/// Get a cached file's content.
pub fn get_cached_file(app: &AppHandle, path: &str) -> Result<Option<Vec<u8>>, AppError> {
    let state = get_conn(app)?;
    let conn = state
        .lock()
        .map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;
    get_cached_file_conn(&conn, path)
}

/// Insert a file entry into the cache.
pub fn insert_file(app: &AppHandle, info: &FileInfo) -> Result<(), AppError> {
    let state = get_conn(app)?;
    let conn = state
        .lock()
        .map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;
    insert_file_conn(&conn, info)
}

/// Remove a file entry from the cache.
pub fn remove_file(app: &AppHandle, path: &str) -> Result<(), AppError> {
    let state = get_conn(app)?;
    let conn = state
        .lock()
        .map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;
    remove_file_conn(&conn, path)
}

/// Clear the entire cache.
pub fn clear_cache(app: &AppHandle) -> Result<(), AppError> {
    let state = get_conn(app)?;
    let conn = state
        .lock()
        .map_err(|e| AppError::DbError(format!("Mutex poisoned: {}", e)))?;
    clear_cache_conn(&conn)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_get_tree_empty_initially() {
        let conn = test_conn();
        assert!(get_tree_conn(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_reindex_populates_default_dirs() {
        let conn = test_conn();
        assert!(reindex_conn(&conn).unwrap());
        let tree = get_tree_conn(&conn).unwrap();
        assert!(tree.iter().any(|f| f.path == "/ext" && f.is_dir));
        assert!(tree.iter().any(|f| f.path == "/ext/subghz"));
    }

    #[test]
    fn test_reindex_clears_previous_entries() {
        let conn = test_conn();
        insert_file_conn(
            &conn,
            &FileInfo {
                path: "/ext/stale.sub".to_string(),
                name: "stale.sub".to_string(),
                size: 1,
                is_dir: false,
                modified: None,
            },
        )
        .unwrap();
        reindex_conn(&conn).unwrap();
        let tree = get_tree_conn(&conn).unwrap();
        assert!(!tree.iter().any(|f| f.path == "/ext/stale.sub"));
    }

    #[test]
    fn test_insert_and_get_tree() {
        let conn = test_conn();
        insert_file_conn(
            &conn,
            &FileInfo {
                path: "/ext/subghz/test.sub".to_string(),
                name: "test.sub".to_string(),
                size: 42,
                is_dir: false,
                modified: None,
            },
        )
        .unwrap();
        let tree = get_tree_conn(&conn).unwrap();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "test.sub");
        assert_eq!(tree[0].size, 42);
    }

    #[test]
    fn test_insert_file_replaces_existing() {
        let conn = test_conn();
        let mut info = FileInfo {
            path: "/ext/test.sub".to_string(),
            name: "test.sub".to_string(),
            size: 1,
            is_dir: false,
            modified: None,
        };
        insert_file_conn(&conn, &info).unwrap();
        info.size = 999;
        insert_file_conn(&conn, &info).unwrap();
        let tree = get_tree_conn(&conn).unwrap();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].size, 999);
    }

    #[test]
    fn test_tree_sorted_dirs_first_then_name() {
        let conn = test_conn();
        for (path, name, is_dir) in [
            ("/ext/b.sub", "b.sub", false),
            ("/ext/a.sub", "a.sub", false),
            ("/ext/zdir", "zdir", true),
        ] {
            insert_file_conn(
                &conn,
                &FileInfo {
                    path: path.to_string(),
                    name: name.to_string(),
                    size: 0,
                    is_dir,
                    modified: None,
                },
            )
            .unwrap();
        }
        let tree = get_tree_conn(&conn).unwrap();
        assert_eq!(tree[0].name, "zdir");
        assert_eq!(tree[1].name, "a.sub");
        assert_eq!(tree[2].name, "b.sub");
    }

    #[test]
    fn test_remove_file() {
        let conn = test_conn();
        let info = FileInfo {
            path: "/ext/test.sub".to_string(),
            name: "test.sub".to_string(),
            size: 0,
            is_dir: false,
            modified: None,
        };
        insert_file_conn(&conn, &info).unwrap();
        remove_file_conn(&conn, "/ext/test.sub").unwrap();
        assert!(get_tree_conn(&conn).unwrap().is_empty());
    }

    #[test]
    fn test_remove_nonexistent_file_is_ok() {
        let conn = test_conn();
        assert!(remove_file_conn(&conn, "/ext/nope.sub").is_ok());
    }

    #[test]
    fn test_cache_and_get_file_content() {
        let conn = test_conn();
        cache_file_conn(&conn, "/ext/test.sub", b"hello world").unwrap();
        let content = get_cached_file_conn(&conn, "/ext/test.sub").unwrap();
        assert_eq!(content, Some(b"hello world".to_vec()));
    }

    #[test]
    fn test_get_cached_file_missing_returns_none() {
        let conn = test_conn();
        assert_eq!(
            get_cached_file_conn(&conn, "/ext/missing.sub").unwrap(),
            None
        );
    }

    #[test]
    fn test_cache_file_replaces_existing_content() {
        let conn = test_conn();
        cache_file_conn(&conn, "/ext/test.sub", b"v1").unwrap();
        cache_file_conn(&conn, "/ext/test.sub", b"v2").unwrap();
        assert_eq!(
            get_cached_file_conn(&conn, "/ext/test.sub").unwrap(),
            Some(b"v2".to_vec())
        );
    }

    #[test]
    fn test_remove_file_also_removes_cached_content() {
        let conn = test_conn();
        let info = FileInfo {
            path: "/ext/test.sub".to_string(),
            name: "test.sub".to_string(),
            size: 0,
            is_dir: false,
            modified: None,
        };
        insert_file_conn(&conn, &info).unwrap();
        cache_file_conn(&conn, "/ext/test.sub", b"data").unwrap();
        remove_file_conn(&conn, "/ext/test.sub").unwrap();
        assert_eq!(get_cached_file_conn(&conn, "/ext/test.sub").unwrap(), None);
    }

    #[test]
    fn test_clear_cache_removes_everything() {
        let conn = test_conn();
        reindex_conn(&conn).unwrap();
        cache_file_conn(&conn, "/ext/test.sub", b"data").unwrap();
        clear_cache_conn(&conn).unwrap();
        assert!(get_tree_conn(&conn).unwrap().is_empty());
        assert_eq!(get_cached_file_conn(&conn, "/ext/test.sub").unwrap(), None);
    }
}
