//! Tests for serial module (varint framing, base64, CLI parsing)

use flipperzero_tool_lib::serial::{encode_varint, read_varint, base64_encode, parse_list_output, FileInfo};
use std::time::Duration;

// === Varint tests ===

#[test]
fn test_varint_zero() {
    let mut buf = Vec::new();
    encode_varint(&mut buf, 0);
    assert_eq!(buf, vec![0]);
    let mut reader = &buf[..];
    assert_eq!(read_varint(&mut reader, Duration::from_secs(1)).unwrap(), 0);
}

#[test]
fn test_varint_single_byte() {
    for val in 1u64..=127 {
        let mut buf = Vec::new();
        encode_varint(&mut buf, val);
        assert_eq!(buf.len(), 1);
        let mut reader = &buf[..];
        assert_eq!(read_varint(&mut reader, Duration::from_secs(1)).unwrap(), val);
    }
}

#[test]
fn test_varint_two_bytes() {
    for val in [128u64, 255, 256, 16383] {
        let mut buf = Vec::new();
        encode_varint(&mut buf, val);
        assert_eq!(buf.len(), 2);
        let mut reader = &buf[..];
        assert_eq!(read_varint(&mut reader, Duration::from_secs(1)).unwrap(), val);
    }
}

#[test]
fn test_varint_three_bytes() {
    for val in [16384u64, 65535, 100000] {
        let mut buf = Vec::new();
        encode_varint(&mut buf, val);
        assert!(buf.len() >= 2 && buf.len() <= 3);
        let mut reader = &buf[..];
        assert_eq!(read_varint(&mut reader, Duration::from_secs(1)).unwrap(), val);
    }
}

#[test]
fn test_varint_large() {
    let val = 2u64.pow(32);
    let mut buf = Vec::new();
    encode_varint(&mut buf, val);
    let mut reader = &buf[..];
    assert_eq!(read_varint(&mut reader, Duration::from_secs(1)).unwrap(), val);
}

#[test]
fn test_varint_max() {
    let val = u64::MAX;
    let mut buf = Vec::new();
    encode_varint(&mut buf, val);
    let mut reader = &buf[..];
    assert_eq!(read_varint(&mut reader, Duration::from_secs(1)).unwrap(), val);
}

#[test]
fn test_varint_roundtrip_random() {
    let values: Vec<u64> = (0..100).map(|i| i * 12345).collect();
    for val in values {
        let mut buf = Vec::new();
        encode_varint(&mut buf, val);
        let mut reader = &buf[..];
        assert_eq!(read_varint(&mut reader, Duration::from_secs(1)).unwrap(), val);
    }
}

// === Base64 tests ===

#[test]
fn test_base64_empty() {
    assert_eq!(base64_encode(b""), "");
}

#[test]
fn test_base64_single_byte() {
    assert_eq!(base64_encode(b"f"), "Zg==");
}

#[test]
fn test_base64_two_bytes() {
    assert_eq!(base64_encode(b"fo"), "Zm8=");
}

#[test]
fn test_base64_three_bytes() {
    assert_eq!(base64_encode(b"foo"), "Zm9v");
}

#[test]
fn test_base64_hello() {
    assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
}

#[test]
fn test_base64_binary() {
    let data = vec![0u8, 1, 2, 255, 128, 64];
    let encoded = base64_encode(&data);
    assert!(!encoded.is_empty());
    assert!(!encoded.contains(' '));
}

#[test]
fn test_base64_large() {
    let data = vec![0xABu8; 1000];
    let encoded = base64_encode(&data);
    assert_eq!(encoded.len(), 1336); // ceil(1000/3)*4
}

// === parse_list_output tests ===

#[test]
fn test_parse_list_empty() {
    let result = parse_list_output("/ext", "").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_parse_list_dirs_first() {
    let output = "[D] dir1
[F] file1.txt 100
[D] dir2
[F] file2.txt 200";
    let result = parse_list_output("/ext", output).unwrap();
    assert_eq!(result.len(), 4);
    // Dirs should come first
    assert!(result[0].is_dir);
    assert!(result[1].is_dir);
    assert!(!result[2].is_dir);
    assert!(!result[3].is_dir);
}

#[test]
fn test_parse_list_file_info() {
    let output = "[F] test.sub 1234";
    let result = parse_list_output("/ext/subghz", output).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "test.sub");
    assert_eq!(result[0].size, 1234);
    assert!(!result[0].is_dir);
    assert_eq!(result[0].path, "/ext/subghz/test.sub");
}

#[test]
fn test_parse_list_dir_info() {
    let output = "[D] subghz";
    let result = parse_list_output("/ext", output).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "subghz");
    assert!(result[0].is_dir);
    assert_eq!(result[0].size, 0);
}

#[test]
fn test_parse_list_with_extra_whitespace() {
    let output = "  [F] file.txt   42  
  [D] mydir  ";
    let result = parse_list_output("/ext", output).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "mydir");
    assert_eq!(result[1].name, "file.txt");
    assert_eq!(result[1].size, 42);
}

#[test]
fn test_parse_list_unknown_lines_ignored() {
    let output = "some random text
[F] file.txt 100
more text";
    let result = parse_list_output("/ext", output).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "file.txt");
}

#[test]
fn test_parse_list_trailing_slash_path() {
    let output = "[F] file.txt 100";
    let result = parse_list_output("/ext/", output).unwrap();
    assert_eq!(result[0].path, "/ext/file.txt");
}
