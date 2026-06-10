//! Tests for parser module - edge cases

use flipperzero_tool_lib::{parse_sub, parse_ir, parse_nfc};

#[test]
fn test_parse_sub_empty() {
    let r = parse_sub("").unwrap();
    assert_eq!(r.file_type, "subghz");
    assert!(r.fields.is_empty());
}

#[test]
fn test_parse_sub_only_comments() {
    let r = parse_sub("# comment
# another
").unwrap();
    assert!(r.fields.is_empty());
}

#[test]
fn test_parse_sub_raw_protocol() {
    let input = "Filetype: Flipper SubGhz Key File
Protocol: RAW
Raw_Data: 100,-200";
    let r = parse_sub(input).unwrap();
    let t = r.fields.iter().find(|f| f["key"] == "type");
    assert!(t.is_some());
    assert_eq!(t.unwrap()["value"], "RAW");
}

#[test]
fn test_parse_sub_all_fields() {
    let input = "Filetype: Flipper SubGhz Key File
Version: 2
Frequency: 315000000
Preset: FuriHalSubGhzPresetOok650Async
Protocol: CAME
Bit: 12
Key: AABB";
    let r = parse_sub(input).unwrap();
    assert!(r.fields.len() >= 6);
}

#[test]
fn test_parse_sub_preview_truncated() {
    let input: String = (0..30).map(|i| format!("Line {}: val
", i)).collect();
    let r = parse_sub(&input).unwrap();
    assert_eq!(r.raw_preview.lines().count(), 20);
}

#[test]
fn test_parse_ir_empty() {
    let r = parse_ir("").unwrap();
    assert_eq!(r.file_type, "infrared");
}

#[test]
fn test_parse_ir_unknown_protocol() {
    let input = "Filetype: IR
Protocol: UnknownProto
Address: 0xFF
Command: 0x00";
    let r = parse_ir(input).unwrap();
    let p = r.fields.iter().find(|f| f["key"] == "protocol");
    assert_eq!(p.unwrap()["value"], "UnknownProto");
}

#[test]
fn test_parse_ir_samsung() {
    let input = "Filetype: IR
Protocol: Samsung
Address: 0x07
Command: 0x02";
    let r = parse_ir(input).unwrap();
    assert_eq!(r.file_type, "infrared");
}

#[test]
fn test_parse_nfc_empty() {
    let r = parse_nfc("").unwrap();
    assert_eq!(r.file_type, "nfc");
}

#[test]
fn test_parse_nfc_mifare_classic() {
    let input = "Filetype: Flipper NFC Key
Device Type: Mifare Classic
UID: 04:1E:23:4A:5B:6C
ATQA: 00 44
SAK: 0x08";
    let r = parse_nfc(input).unwrap();
    let uid = r.fields.iter().find(|f| f["key"] == "uid");
    assert_eq!(uid.unwrap()["value"], "04:1E:23:4A:5B:6C");
}

#[test]
fn test_parse_nfc_ntag() {
    let input = "Filetype: Flipper NFC Key
Device Type: NTAG213
UID: 04:A3:2B:3C";
    let r = parse_nfc(input).unwrap();
    let d = r.fields.iter().find(|f| f["key"] == "device_type");
    assert_eq!(d.unwrap()["value"], "NTAG213");
}

#[test]
fn test_parse_nfc_sak_hex() {
    let input = "Filetype: Flipper NFC Key
SAK: 0x28";
    let r = parse_nfc(input).unwrap();
    assert!(r.fields.iter().any(|f| f["key"] == "sak"));
}

#[test]
fn test_parse_nfc_with_sectors() {
    let input = "Filetype: Flipper NFC Key
Device Type: Mifare Classic
UID: 04:11:22:33
Sector_0: AABB
Sector_1: CCDD";
    let r = parse_nfc(input).unwrap();
    assert!(r.fields.iter().any(|f| f["key"] == "sectors"));
}

#[test]
fn test_all_parsers_unicode() {
    let input = "Filetype: Test
Key: unicode_test_日本語
";
    let _ = parse_sub(input).unwrap();
    let _ = parse_ir(input).unwrap();
    let _ = parse_nfc(input).unwrap();
}

#[test]
fn test_all_parsers_long_input() {
    let val = "A".repeat(10000);
    let input = format!("Filetype: Test
Key: {}
", val);
    let r = parse_sub(&input).unwrap();
    assert_eq!(r.file_type, "subghz");
}
