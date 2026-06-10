//! Tests for commands module - error paths

use flipperzero_tool_lib::*;

#[test]
fn test_list_directory_not_found() {
    let r = list_directory("/nonexistent/path/xyz".to_string());
    assert!(r.is_err());
    match r.unwrap_err() {
        AppError::NotFound(_) => {}
        e => panic!("Expected NotFound, got: {:?}", e),
    }
}

#[test]
fn test_list_directory_file_not_dir() {
    let r = list_directory("/etc/passwd".to_string());
    assert!(r.is_err());
}

#[test]
fn test_find_files_not_found() {
    let r = find_files("/nonexistent".to_string(), "test".to_string());
    assert!(r.is_err());
}

#[test]
fn test_create_file_already_exists() {
    let dir = std::env::temp_dir().join("flipper_test_dup");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("testfile"), "existing").unwrap();
    let r = create_file_from_template(dir.join("testfile").to_string_lossy().to_string(), "sub".to_string());
    assert!(r.is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_move_file_source_not_found() {
    let r = move_file("/nonexistent/src.txt".to_string(), "/tmp/dst.txt".to_string());
    assert!(r.is_err());
}

#[test]
fn test_move_file_dest_exists() {
    let dir = std::env::temp_dir().join("flipper_test_move");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("src.txt"), "data").unwrap();
    std::fs::write(dir.join("dst.txt"), "existing").unwrap();
    let r = move_file(dir.join("src.txt").to_string_lossy().to_string(), dir.join("dst.txt").to_string_lossy().to_string());
    assert!(r.is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_copy_file_source_not_found() {
    let r = copy_file("/nonexistent/src.txt".to_string(), "/tmp/dst.txt".to_string());
    assert!(r.is_err());
}

#[test]
fn test_delete_file_not_found() {
    let r = delete_file("/nonexistent/file.txt".to_string());
    assert!(r.is_err());
}

#[test]
fn test_delete_file_success() {
    let dir = std::env::temp_dir().join("flipper_test_del");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("to_delete.txt"), "data").unwrap();
    let r = delete_file(dir.join("to_delete.txt").to_string_lossy().to_string());
    assert!(r.is_ok());
    assert!(!dir.join("to_delete.txt").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_get_file_content_not_found() {
    let r = get_file_content("/nonexistent/file.txt".to_string());
    assert!(r.is_err());
}

#[test]
fn test_get_file_content_binary() {
    let dir = std::env::temp_dir().join("flipper_test_bin");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("binary.bin"), vec![0xFF, 0xFE, 0x00, 0x01]).unwrap();
    let r = get_file_content(dir.join("binary.bin").to_string_lossy().to_string());
    assert!(r.is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_write_file_content_success() {
    let dir = std::env::temp_dir().join("flipper_test_write");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("test_output.txt");
    let r = write_file_content(path.to_string_lossy().to_string(), "Hello Flipper".to_string());
    assert!(r.is_ok());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "Hello Flipper");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_rename_file_not_found() {
    let r = rename_file("/nonexistent/file.txt".to_string(), "new.txt".to_string());
    assert!(r.is_err());
}

#[test]
fn test_rename_file_success() {
    let dir = std::env::temp_dir().join("flipper_test_rename");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("old.txt"), "data").unwrap();
    let r = rename_file(dir.join("old.txt").to_string_lossy().to_string(), "new.txt".to_string());
    assert!(r.is_ok());
    assert!(!dir.join("old.txt").exists());
    assert!(dir.join("new.txt").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_app_error_display() {
    let errors = vec![
        AppError::NotFound("f".into()),
        AppError::PermissionDenied("p".into()),
        AppError::SerialError("s".into()),
        AppError::ParseError("p".into()),
        AppError::AlreadyExists("a".into()),
        AppError::General("g".into()),
    ];
    for e in errors {
        let msg = format!("{}", e);
        assert!(!msg.is_empty());
    }
}

#[test]
fn test_app_error_from_io() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "nope");
    let e: AppError = io.into();
    match e {
        AppError::NotFound(_) => {}
        _ => panic!("Expected NotFound"),
    }
}

#[test]
fn test_app_error_from_string() {
    let e: AppError = "test".to_string().into();
    match e {
        AppError::General(s) => assert_eq!(s, "test"),
        _ => panic!("Expected General"),
    }
}
