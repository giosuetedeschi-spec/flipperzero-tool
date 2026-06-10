use serde_json::{json, Value};

use super::errors::AppError;

/// A parsed Flipper file with typed fields.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedFile {
    pub file_type: String,
    pub fields: Vec<Value>,
    pub raw_preview: String,
}

// ---------------------------------------------------------------------------
// Key-value helper
// ---------------------------------------------------------------------------

fn parse_key_value(raw: &str) -> Vec<(String, String)> {
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn fields_to_value(fields: &[(String, String)]) -> Vec<Value> {
    fields.iter()
        .map(|(k, v)| json!({"key": k, "value": v}))
        .collect()
}

fn preview(raw: &str) -> String {
    raw.lines().take(20).collect::<Vec<_>>().join("\n")
}

// ---------------------------------------------------------------------------
// Sub-GHz parser (.sub files)
// ---------------------------------------------------------------------------
//
// Example Sub-GHz file:
//   Filetype: Flipper SubGhz Key File
//   Version: 1
//   Frequency: 433920000
//   Preset: FuriHalSubGhzPresetOok650Async
//   Protocol: Princeton
//   Bit: 24
//   Key: 001122334455
//   (for RAW: Raw_Data, Raw_Single_Data, etc.)

pub fn parse_sub(raw: &str) -> Result<ParsedFile, AppError> {
    let kvs = parse_key_value(raw);
    let mut fields = Vec::new();

    let mut filetype = String::new();
    let mut version = 0u32;
    let mut frequency = 0u64;
    let mut preset = String::new();
    let mut protocol = String::new();
    let mut bit = None::<u32>;
    let mut key = String::new();
    let mut raw_data = false;

    for (k, v) in &kvs {
        match k.as_str() {
            "Filetype" => { filetype = v.clone(); }
            "Version" => { version = v.parse().unwrap_or(0); }
            "Frequency" => {
                frequency = v.parse().map_err(|_| {
                    AppError::ParseError(format!("Invalid frequency: {}", v))
                })?;
            }
            "Preset" => { preset = v.clone(); }
            "Protocol" => { protocol = v.clone(); }
            "Bit" => {
                bit = Some(v.parse().map_err(|_| {
                    AppError::ParseError(format!("Invalid bit count: {}", v))
                })?);
            }
            "Key" => { key = v.clone(); }
            "Raw_Data" | "Raw_Single_Data" => { raw_data = true; }
            _ => {}
        }
    }

    fields.push(json!({"key": "filetype", "value": filetype}));
    fields.push(json!({"key": "version", "value": version}));
    if frequency > 0 {
        fields.push(json!({"key": "frequency", "value": frequency}));
    }
    if !preset.is_empty() {
        fields.push(json!({"key": "preset", "value": preset}));
    }
    if !protocol.is_empty() {
        fields.push(json!({"key": "protocol", "value": protocol}));
    }
    if let Some(b) = bit {
        fields.push(json!({"key": "bit", "value": b}));
    }
    if !key.is_empty() {
        fields.push(json!({"key": "key", "value": key}));
    }
    if raw_data {
        fields.push(json!({"key": "type", "value": "RAW"}));
    }

    // Add any remaining unknown fields
    for (k, v) in &kvs {
        let known = ["Filetype", "Version", "Frequency", "Preset", "Protocol",
                     "Bit", "Key", "Raw_Data", "Raw_Single_Data"];
        if !known.contains(&k.as_str()) {
            fields.push(json!({"key": k, "value": v}));
        }
    }

    Ok(ParsedFile {
        file_type: "subghz".to_string(),
        fields,
        raw_preview: preview(raw),
    })
}

// ---------------------------------------------------------------------------
// Infrared parser (.ir files)
// ---------------------------------------------------------------------------
//
// Example IR file:
//   Filetype: IR
//   Version: 1
//   Protocol: NEC
//   Address: 0x00
//   Command: 0x45
//   (or for raw: Raw_Data with comma-separated values)

pub fn parse_ir(raw: &str) -> Result<ParsedFile, AppError> {
    let kvs = parse_key_value(raw);
    let mut fields = Vec::new();

    let mut filetype = String::new();
    let mut version = 0u32;
    let mut protocol = String::new();
    let mut address = String::new();
    let mut command = String::new();
    let mut buttons: Vec<Value> = Vec::new();
    let mut current_btn = String::new();
    let mut btn_protocol = String::new();
    let mut btn_address = String::new();
    let btn_command = String::new();
    let mut raw_data = false;

    for (k, v) in &kvs {
        match k.as_str() {
            "Filetype" => { filetype = v.clone(); }
            "Version" => { version = v.parse().unwrap_or(0); }
            "Protocol" => { protocol = v.clone(); }
            "Address" => { address = v.clone(); }
            "Command" => { command = v.clone(); }
            "Raw_Data" => { raw_data = true; }
            k if k.starts_with("Button_") => {
                // Multi-button format: Button_1, Button_1_Protocol, etc.
                current_btn = v.clone();
            }
            k if k.contains("Protocol") && !k.contains("Filetype") => {
                btn_protocol = v.clone();
            }
            k if k.contains("Address") => {
                btn_address = v.clone();
            }
            k if k.contains("Command") && k != "Command" => {
                let _btn_command = v.clone();
                if !current_btn.is_empty() {
                    buttons.push(json!({
                        "name": current_btn,
                        "protocol": btn_protocol,
                        "address": btn_address,
                        "command": btn_command,
                    }));
                }
            }
            _ => {}
        }
    }

    fields.push(json!({"key": "filetype", "value": filetype}));
    fields.push(json!({"key": "version", "value": version}));
    if !protocol.is_empty() {
        fields.push(json!({"key": "protocol", "value": protocol}));
    }
    if !address.is_empty() {
        fields.push(json!({"key": "address", "value": address}));
    }
    if !command.is_empty() {
        fields.push(json!({"key": "command", "value": command}));
    }
    if raw_data {
        fields.push(json!({"key": "type", "value": "RAW"}));
    }
    if !buttons.is_empty() {
        fields.push(json!({"key": "buttons", "value": buttons}));
    }

    Ok(ParsedFile {
        file_type: "infrared".to_string(),
        fields,
        raw_preview: preview(raw),
    })
}

// ---------------------------------------------------------------------------
// NFC parser (.nfc files)
// ---------------------------------------------------------------------------
//
// Example NFC file:
//   Filetype: Flipper NFC Key
//   Version: 1
//   Device Type: Mifare Classic
//   UID: 04:1E:23:4A:5B:6C
//   ATQA: 00 44
//   SAK: 08

pub fn parse_nfc(raw: &str) -> Result<ParsedFile, AppError> {
    let kvs = parse_key_value(raw);
    let mut fields = Vec::new();

    let mut filetype = String::new();
    let mut version = 0u32;
    let mut device_type = String::new();
    let mut uid = String::new();
    let mut atqa = String::new();
    let mut sak = 0u8;
    let mut sectors: Vec<Value> = Vec::new();

    for (k, v) in &kvs {
        match k.as_str() {
            "Filetype" => { filetype = v.clone(); }
            "Version" => { version = v.parse().unwrap_or(0); }
            "Device Type" | "DeviceType" => { device_type = v.clone(); }
            "UID" => { uid = v.clone(); }
            "ATQA" => { atqa = v.clone(); }
            "SAK" => {
                sak = u8::from_str_radix(v.trim_start_matches("0x"), 16)
                    .unwrap_or_else(|_| v.parse().unwrap_or(0));
            }
            k if k.starts_with("Sector") || k.starts_with("Block") => {
                sectors.push(json!({"key": k, "value": v}));
            }
            _ => {}
        }
    }

    fields.push(json!({"key": "filetype", "value": filetype}));
    fields.push(json!({"key": "version", "value": version}));
    if !device_type.is_empty() {
        fields.push(json!({"key": "device_type", "value": device_type}));
    }
    if !uid.is_empty() {
        fields.push(json!({"key": "uid", "value": uid}));
    }
    if !atqa.is_empty() {
        fields.push(json!({"key": "atqa", "value": atqa}));
    }
    if sak > 0 {
        fields.push(json!({"key": "sak", "value": sak}));
    }
    if !sectors.is_empty() {
        fields.push(json!({"key": "sectors", "value": sectors}));
    }

    // Add unknown fields
    let known = ["Filetype", "Version", "Device Type", "DeviceType",
                 "UID", "ATQA", "SAK"];
    for (k, v) in &kvs {
        if !known.contains(&k.as_str()) && !k.starts_with("Sector") && !k.starts_with("Block") {
            fields.push(json!({"key": k, "value": v}));
        }
    }

    Ok(ParsedFile {
        file_type: "nfc".to_string(),
        fields,
        raw_preview: preview(raw),
    })
}

// ---------------------------------------------------------------------------
// Generic fallback parser
// ---------------------------------------------------------------------------

pub fn parse_generic(raw: &str) -> ParsedFile {
    let kvs = parse_key_value(raw);
    ParsedFile {
        file_type: "generic".to_string(),
        fields: fields_to_value(&kvs),
        raw_preview: preview(raw),
    }
}
