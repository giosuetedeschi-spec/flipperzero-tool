use serde_json::{json, Value};

use super::errors::AppError;

use super::commands::ParsedFile;

fn parse_key_value(raw: &str) -> Vec<Value> {
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                Some(json!({"key": parts[0].trim(), "value": parts[1].trim()}))
            } else {
                None
            }
        })
        .collect()
}

pub fn parse_sub(raw: &str) -> Result<ParsedFile, AppError> {
    let fields = parse_key_value(raw);
    Ok(ParsedFile {
        file_type: "subghz".to_string(),
        fields,
        raw_preview: raw.lines().take(20).collect::<Vec<_>>().join("\n"),
    })
}

pub fn parse_ir(raw: &str) -> Result<ParsedFile, AppError> {
    let fields = parse_key_value(raw);
    Ok(ParsedFile {
        file_type: "infrared".to_string(),
        fields,
        raw_preview: raw.lines().take(20).collect::<Vec<_>>().join("\n"),
    })
}

pub fn parse_nfc(raw: &str) -> Result<ParsedFile, AppError> {
    let fields = parse_key_value(raw);
    Ok(ParsedFile {
        file_type: "nfc".to_string(),
        fields,
        raw_preview: raw.lines().take(20).collect::<Vec<_>>().join("\n"),
    })
}
