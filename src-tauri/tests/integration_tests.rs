//! Integration tests for flipperzero-tool backend

use flipperzero_tool_lib::{list_directory, find_files, create_file_from_template, move_file};
use std::fs;
use std::path::Path;

#[test]
fn test_list_directory_empty() {
    let dir = std::env::temp_dir().join("flipper_test_empty");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    let result = list_directory(dir.to_string_lossy().to_string()).unwrap();
    assert_eq!(result.len(), 0);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_list_directory_with_files() {
    let dir = std::env::temp_dir().join("flipper_test_files");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("test.txt"), "hello").unwrap();
    fs::write(dir.join("test.sub"), "Filetype: Test").unwrap();
    let result = list_directory(dir.to_string_lossy().to_string()).unwrap();
    assert_eq!(result.len(), 2);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_list_directory_not_found() {
    let result = list_directory("/nonexistent/path/xyz".to_string());
    assert!(result.is_err());
}

#[test]
fn test_find_files() {
    let dir = std::env::temp_dir().join("flipper_test_find");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    fs::create_dir(dir.join("sub")).unwrap();
    fs::write(dir.join("a.txt"), "").unwrap();
    fs::write(dir.join("sub/b.txt"), "").unwrap();
    fs::write(dir.join("c.sub"), "").unwrap();
    let results = find_files(dir.to_string_lossy().to_string(), "txt".to_string()).unwrap();
    assert_eq!(results.len(), 2);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_create_file_from_template() {
    let dir = std::env::temp_dir().join("flipper_test_create");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    let path = dir.join("testfile");
    let result = create_file_from_template(path.to_string_lossy().to_string(), "sub".to_string()).unwrap();
    assert!(Path::new(&result).exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_create_file_already_exists() {
    let dir = std::env::temp_dir().join("flipper_test_exists");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    let path = dir.join("exists");
    create_file_from_template(path.to_string_lossy().to_string(), "sub".to_string()).unwrap();
    let result = create_file_from_template(path.to_string_lossy().to_string(), "sub".to_string());
    assert!(result.is_err());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_move_file_success() {
    let dir = std::env::temp_dir().join("flipper_test_move");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    let src = dir.join("src.txt");
    let dst = dir.join("dst.txt");
    fs::write(&src, "data").unwrap();
    move_file(src.to_string_lossy().to_string(), dst.to_string_lossy().to_string()).unwrap();
    assert!(!src.exists());
    assert!(dst.exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_move_file_already_exists() {
    let dir = std::env::temp_dir().join("flipper_test_move_exists");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    let src = dir.join("src.txt");
    let dst = dir.join("dst.txt");
    fs::write(&src, "data").unwrap();
    fs::write(&dst, "existing").unwrap();
    let result = move_file(src.to_string_lossy().to_string(), dst.to_string_lossy().to_string());
    assert!(result.is_err());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_parse_sub() {
    let raw = "Filetype: Flipper SubGhz Key File\nVersion: 1\nFrequency: 433920000\nProtocol: Princeton\nBit: 24\nKey: AABBCCDD";
    let result = flipperzero_tool_lib::parse_sub(raw).unwrap();
    assert_eq!(result.file_type, "subghz");
    assert!(result.fields.len() >= 4);
}

#[test]
fn test_parse_ir() {
    let raw = "Filetype: IR\nVersion: 1\nProtocol: NEC\nAddress: 0x00\nCommand: 0x45";
    let result = flipperzero_tool_lib::parse_ir(raw).unwrap();
    assert_eq!(result.file_type, "infrared");
}

#[test]
fn test_parse_nfc() {
    let raw = "Filetype: Flipper NFC Key\nVersion: 1\nDevice Type: Mifare Classic\nUID: 04:1E:23:4A:5B:6C\nATQA: 00 44\nSAK: 0x08";
    let result = flipperzero_tool_lib::parse_nfc(raw).unwrap();
    assert_eq!(result.file_type, "nfc");
}
