use serde_json::{Value, json};

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
    fields
        .iter()
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
            "Filetype" => {
                filetype = v.clone();
            }
            "Version" => {
                version = v.parse().unwrap_or(0);
            }
            "Frequency" => {
                frequency = v
                    .parse()
                    .map_err(|_| AppError::ParseError(format!("Invalid frequency: {}", v)))?;
            }
            "Preset" => {
                preset = v.clone();
            }
            "Protocol" => {
                protocol = v.clone();
            }
            "Bit" => {
                bit = Some(
                    v.parse()
                        .map_err(|_| AppError::ParseError(format!("Invalid bit count: {}", v)))?,
                );
            }
            "Key" => {
                key = v.clone();
            }
            "Raw_Data" | "Raw_Single_Data" => {
                raw_data = true;
            }
            _ => {}
        }
    }

    if !filetype.is_empty() {
        fields.push(json!({"key": "filetype", "value": filetype}));
    }
    if version > 0 {
        fields.push(json!({"key": "version", "value": version}));
    }
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
        let known = [
            "Filetype",
            "Version",
            "Frequency",
            "Preset",
            "Protocol",
            "Bit",
            "Key",
            "Raw_Data",
            "Raw_Single_Data",
        ];
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
            "Filetype" => {
                filetype = v.clone();
            }
            "Version" => {
                version = v.parse().unwrap_or(0);
            }
            "Protocol" => {
                protocol = v.clone();
            }
            "Address" => {
                address = v.clone();
            }
            "Command" => {
                command = v.clone();
            }
            "Raw_Data" => {
                raw_data = true;
            }
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
            "Filetype" => {
                filetype = v.clone();
            }
            "Version" => {
                version = v.parse().unwrap_or(0);
            }
            "Device Type" | "DeviceType" => {
                device_type = v.clone();
            }
            "UID" => {
                uid = v.clone();
            }
            "ATQA" => {
                atqa = v.clone();
            }
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
    let known = [
        "Filetype",
        "Version",
        "Device Type",
        "DeviceType",
        "UID",
        "ATQA",
        "SAK",
    ];
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

// ---------------------------------------------------------------------------
// Structured parser types (P3)
// ---------------------------------------------------------------------------

/// Parsed Sub-GHz file with typed fields.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubGhzFile {
    pub filetype: String,
    pub version: u32,
    pub frequency: u64,
    pub preset: String,
    pub protocol: String,
    pub bit: Option<u32>,
    pub key: String,
    pub is_raw: bool,
    pub extra: Vec<(String, String)>,
}

impl SubGhzFile {
    pub fn display_frequency(&self) -> String {
        if self.frequency >= 1_000_000 {
            format!("{:.2} MHz", self.frequency as f64 / 1_000_000.0)
        } else if self.frequency >= 1_000 {
            format!("{:.1} kHz", self.frequency as f64 / 1_000.0)
        } else {
            format!("{} Hz", self.frequency)
        }
    }
}

impl From<SubGhzFile> for ParsedFile {
    fn from(s: SubGhzFile) -> Self {
        let mut fields = Vec::new();
        let val = serde_json::json!({
            "filetype": s.filetype,
            "version": s.version,
            "frequency": s.frequency,
            "frequency_display": s.display_frequency(),
            "preset": s.preset,
            "protocol": s.protocol,
            "bit": s.bit,
            "key": s.key,
            "is_raw": s.is_raw,
        });
        fields.push(val);
        ParsedFile {
            file_type: "subghz".to_string(),
            fields,
            raw_preview: format!(
                "Frequency: {} | Protocol: {}",
                s.display_frequency(),
                s.protocol
            ),
        }
    }
}

/// Parsed IR file with typed fields.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IrFile {
    pub filetype: String,
    pub version: u32,
    pub protocol: String,
    pub address: String,
    pub command: String,
    pub buttons: Vec<IrButton>,
    pub is_raw: bool,
    pub extra: Vec<(String, String)>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IrButton {
    pub name: String,
    pub protocol: String,
    pub address: String,
    pub command: String,
}

impl From<IrFile> for ParsedFile {
    fn from(ir: IrFile) -> Self {
        let val = serde_json::json!({
            "filetype": ir.filetype,
            "version": ir.version,
            "protocol": ir.protocol,
            "address": ir.address,
            "command": ir.command,
            "buttons": ir.buttons,
            "is_raw": ir.is_raw,
        });
        ParsedFile {
            file_type: "infrared".to_string(),
            fields: vec![val],
            raw_preview: format!("Protocol: {} | Buttons: {}", ir.protocol, ir.buttons.len()),
        }
    }
}

/// Parsed NFC file with typed fields.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NfcFile {
    pub filetype: String,
    pub version: u32,
    pub device_type: String,
    pub uid: String,
    pub atqa: String,
    pub sak: u8,
    pub sectors: Vec<NfcSector>,
    pub extra: Vec<(String, String)>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NfcSector {
    pub index: u32,
    pub blocks: Vec<NfcBlock>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NfcBlock {
    pub index: u32,
    pub data: String,
    pub readable: bool,
}

impl From<NfcFile> for ParsedFile {
    fn from(nfc: NfcFile) -> Self {
        let val = serde_json::json!({
            "filetype": nfc.filetype,
            "version": nfc.version,
            "device_type": nfc.device_type,
            "uid": nfc.uid,
            "atqa": nfc.atqa,
            "sak": nfc.sak,
            "sectors": nfc.sectors,
        });
        ParsedFile {
            file_type: "nfc".to_string(),
            fields: vec![val],
            raw_preview: format!("Type: {} | UID: {}", nfc.device_type, nfc.uid),
        }
    }
}

impl ParsedFile {
    /// Parse SubGhz into structured type.
    pub fn parse_sub_struct(raw: &str) -> Result<SubGhzFile, AppError> {
        let kvs = parse_key_value(raw);
        let mut filetype = String::new();
        let mut version = 0u32;
        let mut frequency = 0u64;
        let mut preset = String::new();
        let mut protocol = String::new();
        let mut bit = None::<u32>;
        let mut key = String::new();
        let mut is_raw = false;
        let mut extra = Vec::new();

        for (k, v) in &kvs {
            match k.as_str() {
                "Filetype" => filetype = v.clone(),
                "Version" => version = v.parse().unwrap_or(0),
                "Frequency" => {
                    frequency = v
                        .parse()
                        .map_err(|_| AppError::ParseError(format!("Invalid frequency: {}", v)))?
                }
                "Preset" => preset = v.clone(),
                "Protocol" => protocol = v.clone(),
                "Bit" => {
                    bit = Some(
                        v.parse()
                            .map_err(|_| AppError::ParseError(format!("Invalid bit: {}", v)))?,
                    )
                }
                "Key" => key = v.clone(),
                "Raw_Data" | "Raw_Single_Data" => is_raw = true,
                _ => extra.push((k.clone(), v.clone())),
            }
        }

        Ok(SubGhzFile {
            filetype,
            version,
            frequency,
            preset,
            protocol,
            bit,
            key,
            is_raw,
            extra,
        })
    }

    /// Parse IR into structured type.
    pub fn parse_ir_struct(raw: &str) -> Result<IrFile, AppError> {
        let kvs = parse_key_value(raw);
        let mut filetype = String::new();
        let mut version = 0u32;
        let mut protocol = String::new();
        let mut address = String::new();
        let mut command = String::new();
        let mut buttons = Vec::new();
        let mut is_raw = false;
        let mut extra = Vec::new();

        // Collect button data
        let mut btn_map: std::collections::HashMap<String, IrButton> =
            std::collections::HashMap::new();
        let known_ir_keys = [
            "Filetype", "Version", "Protocol", "Address", "Command", "Raw_Data",
        ];

        for (k, v) in &kvs {
            match k.as_str() {
                "Filetype" => filetype = v.clone(),
                "Version" => version = v.parse().unwrap_or(0),
                "Protocol" => protocol = v.clone(),
                "Address" => address = v.clone(),
                "Command" => command = v.clone(),
                "Raw_Data" => is_raw = true,
                _ => {
                    if k.starts_with("Button_") {
                        let btn_name = v.clone();
                        btn_map.entry(btn_name.clone()).or_insert(IrButton {
                            name: btn_name,
                            protocol: String::new(),
                            address: String::new(),
                            command: String::new(),
                        });
                    } else if k.contains("_Protocol") {
                        // Extract button name from key like "Button_1_Protocol"
                        let parts: Vec<&str> = k.split('_').collect();
                        if parts.len() >= 3 {
                            let btn_key = format!("Button_{}", parts[1]);
                            if let Some(btn) = btn_map.get_mut(&btn_key) {
                                btn.protocol = v.clone();
                            }
                        }
                    } else if k.contains("_Address") {
                        let parts: Vec<&str> = k.split('_').collect();
                        if parts.len() >= 3 {
                            let btn_key = format!("Button_{}", parts[1]);
                            if let Some(btn) = btn_map.get_mut(&btn_key) {
                                btn.address = v.clone();
                            }
                        }
                    } else if k.contains("_Command") && k != "Command" {
                        let parts: Vec<&str> = k.split('_').collect();
                        if parts.len() >= 3 {
                            let btn_key = format!("Button_{}", parts[1]);
                            if let Some(btn) = btn_map.get_mut(&btn_key) {
                                btn.command = v.clone();
                            }
                        }
                    } else if !known_ir_keys.contains(&k.as_str()) {
                        extra.push((k.clone(), v.clone()));
                    }
                }
            }
        }

        buttons.extend(btn_map.into_values());

        Ok(IrFile {
            filetype,
            version,
            protocol,
            address,
            command,
            buttons,
            is_raw,
            extra,
        })
    }

    /// Parse NFC into structured type.
    pub fn parse_nfc_struct(raw: &str) -> Result<NfcFile, AppError> {
        let kvs = parse_key_value(raw);
        let mut filetype = String::new();
        let mut version = 0u32;
        let mut device_type = String::new();
        let mut uid = String::new();
        let mut atqa = String::new();
        let mut sak = 0u8;
        let sectors = Vec::new();
        let mut extra = Vec::new();

        for (k, v) in &kvs {
            match k.as_str() {
                "Filetype" => filetype = v.clone(),
                "Version" => version = v.parse().unwrap_or(0),
                "Device Type" | "DeviceType" => device_type = v.clone(),
                "UID" => uid = v.clone(),
                "ATQA" => atqa = v.clone(),
                "SAK" => {
                    sak = u8::from_str_radix(v.trim_start_matches("0x"), 16)
                        .unwrap_or_else(|_| v.parse().unwrap_or(0));
                }
                _ => {
                    if k.starts_with("Sector") || k.starts_with("Block") {
                        extra.push((k.clone(), v.clone()));
                    }
                }
            }
        }

        Ok(NfcFile {
            filetype,
            version,
            device_type,
            uid,
            atqa,
            sak,
            sectors,
            extra,
        })
    }
}

// ---------------------------------------------------------------------------
// LF RFID parser (.rfid files)
// ---------------------------------------------------------------------------
//
// 125 kHz badges: the kind used by building entrances and company gates. The
// payload is short -- these tags hold little more than an identifier.
//
// Example:
//   Filetype: Flipper RFID key
//   Version: 1
//   Key type: EM4100
//   Data: 12 34 56 78 90

/// Parse a `.rfid` file into the loose field representation.
pub fn parse_rfid(raw: &str) -> Result<ParsedFile, AppError> {
    let kvs = parse_key_value(raw);
    let mut fields = Vec::new();

    for (k, v) in &kvs {
        match k.as_str() {
            "Filetype" => fields.push(Value::String(format!("filetype: {}", v))),
            "Version" => fields.push(Value::String(format!("version: {}", v))),
            // The Flipper writes "Key type"; some tools emit "Key_type".
            "Key type" | "Key_type" | "KeyType" => {
                fields.push(Value::String(format!("key_type: {}", v)))
            }
            "Data" => fields.push(Value::String(format!("data: {}", v))),
            _ => fields.push(Value::String(format!("{}: {}", k, v))),
        }
    }

    Ok(ParsedFile {
        file_type: "rfid".to_string(),
        fields,
        raw_preview: preview(raw),
    })
}

// ---------------------------------------------------------------------------
// iButton parser (.ibtn files)
// ---------------------------------------------------------------------------
//
// 1-Wire contact keys -- the round metal keys used on intercoms.
//
// Example:
//   Filetype: Flipper iButton key
//   Version: 1
//   Key type: Dallas
//   Data: 01 02 03 04 05 06 07 08

/// Parse an `.ibtn` file into the loose field representation.
pub fn parse_ibtn(raw: &str) -> Result<ParsedFile, AppError> {
    let kvs = parse_key_value(raw);
    let mut fields = Vec::new();

    for (k, v) in &kvs {
        match k.as_str() {
            "Filetype" => fields.push(Value::String(format!("filetype: {}", v))),
            "Version" => fields.push(Value::String(format!("version: {}", v))),
            "Key type" | "Key_type" | "KeyType" => {
                fields.push(Value::String(format!("key_type: {}", v)))
            }
            "Data" => fields.push(Value::String(format!("data: {}", v))),
            _ => fields.push(Value::String(format!("{}: {}", k, v))),
        }
    }

    Ok(ParsedFile {
        file_type: "ibtn".to_string(),
        fields,
        raw_preview: preview(raw),
    })
}

/// A parsed `.rfid` file.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RfidFile {
    pub filetype: String,
    pub version: u32,
    /// Tag protocol, e.g. `EM4100`, `HIDProx`, `Indala26`.
    pub key_type: String,
    /// Identifier bytes as written in the file, space separated.
    pub data: String,
    pub extra: Vec<(String, String)>,
}

/// A parsed `.ibtn` file.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct IButtonFile {
    pub filetype: String,
    pub version: u32,
    /// 1-Wire family, e.g. `Dallas`, `Cyfral`, `Metakom`.
    pub key_type: String,
    pub data: String,
    pub extra: Vec<(String, String)>,
}

/// Shared shape of the two key-file formats.
///
/// `.rfid` and `.ibtn` are the same four fields over different radios, so they
/// are parsed once rather than twice with the field names copied.
fn parse_key_file(raw: &str) -> (String, u32, String, String, Vec<(String, String)>) {
    let mut filetype = String::new();
    let mut version = 0u32;
    let mut key_type = String::new();
    let mut data = String::new();
    let mut extra = Vec::new();

    for (k, v) in parse_key_value(raw) {
        match k.as_str() {
            "Filetype" => filetype = v,
            "Version" => version = v.parse().unwrap_or(0),
            "Key type" | "Key_type" | "KeyType" => key_type = v,
            "Data" => data = v,
            _ => extra.push((k, v)),
        }
    }

    (filetype, version, key_type, data, extra)
}

/// Parse a `.rfid` file into a typed struct.
pub fn parse_rfid_struct(raw: &str) -> Result<RfidFile, AppError> {
    let (filetype, version, key_type, data, extra) = parse_key_file(raw);
    Ok(RfidFile {
        filetype,
        version,
        key_type,
        data,
        extra,
    })
}

/// Parse an `.ibtn` file into a typed struct.
pub fn parse_ibtn_struct(raw: &str) -> Result<IButtonFile, AppError> {
    let (filetype, version, key_type, data, extra) = parse_key_file(raw);
    Ok(IButtonFile {
        filetype,
        version,
        key_type,
        data,
        extra,
    })
}

#[cfg(test)]
mod key_file_tests {
    use super::*;

    const RFID: &str =
        "Filetype: Flipper RFID key\nVersion: 1\nKey type: EM4100\nData: 12 34 56 78 90\n";
    const IBTN: &str = "Filetype: Flipper iButton key\nVersion: 1\nKey type: Dallas\nData: 01 02 03 04 05 06 07 08\n";

    #[test]
    fn parses_an_rfid_badge() {
        let parsed = parse_rfid_struct(RFID).unwrap();
        assert_eq!(parsed.filetype, "Flipper RFID key");
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.key_type, "EM4100");
        assert_eq!(parsed.data, "12 34 56 78 90");
    }

    #[test]
    fn parses_an_ibutton_key() {
        let parsed = parse_ibtn_struct(IBTN).unwrap();
        assert_eq!(parsed.filetype, "Flipper iButton key");
        assert_eq!(parsed.key_type, "Dallas");
        assert_eq!(parsed.data, "01 02 03 04 05 06 07 08");
    }

    #[test]
    fn accepts_the_underscore_spelling_of_key_type() {
        // Third-party tools write Key_type; the Flipper writes "Key type".
        let parsed = parse_rfid_struct("Key_type: HIDProx\nData: AA BB\n").unwrap();
        assert_eq!(parsed.key_type, "HIDProx");
    }

    #[test]
    fn unknown_lines_are_kept_rather_than_dropped() {
        let parsed = parse_ibtn_struct("Key type: Cyfral\nNote: front door\n").unwrap();
        assert_eq!(
            parsed.extra,
            vec![("Note".to_string(), "front door".to_string())]
        );
    }

    #[test]
    fn empty_input_yields_empty_fields_not_an_error() {
        let parsed = parse_rfid_struct("").unwrap();
        assert_eq!(parsed.version, 0);
        assert!(parsed.key_type.is_empty());
        assert!(parsed.data.is_empty());
    }

    #[test]
    fn a_malformed_version_does_not_abort_the_parse() {
        let parsed = parse_rfid_struct("Version: not-a-number\nKey type: EM4100\n").unwrap();
        assert_eq!(parsed.version, 0);
        assert_eq!(parsed.key_type, "EM4100", "later fields still parse");
    }

    #[test]
    fn loose_parsers_tag_their_file_type() {
        assert_eq!(parse_rfid(RFID).unwrap().file_type, "rfid");
        assert_eq!(parse_ibtn(IBTN).unwrap().file_type, "ibtn");
    }

    #[test]
    fn loose_parsers_keep_every_line() {
        assert_eq!(parse_rfid(RFID).unwrap().fields.len(), 4);
        assert_eq!(parse_ibtn(IBTN).unwrap().fields.len(), 4);
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let parsed = parse_rfid_struct("# a comment\n\nKey type: Indala26\n").unwrap();
        assert_eq!(parsed.key_type, "Indala26");
        assert!(parsed.extra.is_empty());
    }
}
