//! Reverse Engineer module.
//!
//! Analyzes raw byte streams to identify patterns, protocols, and structure.
//! Provides:
//!   - Byte pattern analysis (repeating sequences, headers, delimiters)
//!   - Protocol fingerprinting (compare against known signatures)
//!   - Entropy calculation (encrypted vs plaintext detection)
//!   - Structure inference (field boundaries, length fields)
//!   - Protobuf candidate generation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatternMatch {
    pub offset: usize,
    pub length: usize,
    pub pattern: Vec<u8>,
    pub confidence: f64,  // 0.0 - 1.0
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProtocolFingerprint {
    pub name: String,
    pub signature: Vec<u8>,
    pub offset: usize,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalysisResult {
    pub entropy: f64,
    pub total_bytes: usize,
    pub unique_bytes: usize,
    pub patterns: Vec<PatternMatch>,
    pub matched_protocols: Vec<ProtocolFingerprint>,
    pub inferred_structure: Vec<FieldCandidate>,
    pub hex_preview: String,
    pub ascii_preview: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FieldCandidate {
    pub offset: usize,
    pub length: usize,
    pub field_type: String,  // "header", "length", "payload", "checksum", "unknown"
    pub confidence: f64,
    pub value_hex: String,
    pub value_dec: Option<u64>,
}

// ---------------------------------------------------------------------------
// Known protocol signatures
// ---------------------------------------------------------------------------

fn known_protocols() -> Vec<ProtocolFingerprint> {
    vec![
        ProtocolFingerprint {
            name: "NEC IR".to_string(),
            signature: vec![0x00, 0xFF],
            offset: 0,
            description: "NEC infrared protocol (32-bit, start with 00 FF)".to_string(),
        },
        ProtocolFingerprint {
            name: "NEC IR (repeat)".to_string(),
            signature: vec![0x00, 0xFF, 0x00, 0x00],
            offset: 0,
            description: "NEC repeat code".to_string(),
        },
        ProtocolFingerprint {
            name: "RC5".to_string(),
            signature: vec![0x36],
            offset: 0,
            description: "RC5 protocol (start bit 0x36)".to_string(),
        },
        ProtocolFingerprint {
            name: "RC6".to_string(),
            signature: vec![0x37],
            offset: 0,
            description: "RC6 protocol".to_string(),
        },
        ProtocolFingerprint {
            name: "Sony SIRC".to_string(),
            signature: vec![0x01],
            offset: 0,
            description: "Sony SIRC (12-bit)".to_string(),
        },
        ProtocolFingerprint {
            name: "Sub-GHz OOK".to_string(),
            signature: vec![0xAA, 0xAA],
            offset: 0,
            description: "Sub-GHz OOK preamble (AA AA)".to_string(),
        },
        ProtocolFingerprint {
            name: "NFC NDEF".to_string(),
            signature: vec![0x03],
            offset: 0,
            description: "NFC NDEF message (starts with 0x03)".to_string(),
        },
        ProtocolFingerprint {
            name: "Protobuf varint".to_string(),
            signature: vec![],
            offset: 0,
            description: "Protobuf varint encoding detected".to_string(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Entropy calculation
// ---------------------------------------------------------------------------

/// Calculate Shannon entropy of a byte slice.
/// Returns value between 0.0 (all same byte) and 8.0 (perfectly random).
pub fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut freq = [0u64; 256];
    for &b in data {
        freq[b as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in &freq {
        if count == 0 {
            continue;
        }
        let p = count as f64 / len;
        entropy -= p * p.log2();
    }

    entropy
}

// ---------------------------------------------------------------------------
// Pattern detection
// ---------------------------------------------------------------------------

/// Find repeating byte patterns in data.
pub fn find_patterns(data: &[u8], min_len: usize, max_len: usize) -> Vec<PatternMatch> {
    let mut patterns = Vec::new();
    let n = data.len();

    if n < min_len * 2 {
        return patterns;
    }

    for pattern_len in min_len..=max_len.min(n / 2) {
        for start in 0..=n - pattern_len * 2 {
            let candidate = &data[start..start + pattern_len];
            let mut match_count = 0;
            let mut offset = start + pattern_len;

            while offset + pattern_len <= n {
                if &data[offset..offset + pattern_len] == candidate {
                    match_count += 1;
                    offset += pattern_len;
                } else {
                    break;
                }
            }

            if match_count >= 2 {
                let confidence = (match_count as f64 * pattern_len as f64) / n as f64;
                if confidence > 0.1 {
                    patterns.push(PatternMatch {
                        offset: start,
                        length: pattern_len,
                        pattern: candidate.to_vec(),
                        confidence: confidence.min(1.0),
                        description: format!(
                            "Repeating {}-byte pattern, {} occurrences",
                            pattern_len, match_count
                        ),
                    });
                }
            }
        }
    }

    // Deduplicate: keep highest confidence for each offset
    patterns.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    let mut seen = std::collections::HashSet::new();
    patterns.retain(|p| seen.insert(p.offset));

    patterns.truncate(20); // Limit results
    patterns
}

// ---------------------------------------------------------------------------
// Protocol fingerprinting
// ---------------------------------------------------------------------------

/// Compare data against known protocol signatures.
pub fn fingerprint_protocols(data: &[u8]) -> Vec<ProtocolFingerprint> {
    let mut matches = Vec::new();

    for proto in known_protocols() {
        if proto.signature.is_empty() {
            // Special case: protobuf varint detection
            if detect_protobuf_varint(data) {
                matches.push(proto);
            }
            continue;
        }

        if data.len() >= proto.offset + proto.signature.len() {
            if &data[proto.offset..proto.offset + proto.signature.len()] == proto.signature.as_slice() {
                matches.push(proto);
            }
        }
    }

    matches
}

/// Detect protobuf varint encoding patterns.
fn detect_protobuf_varint(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    let mut varint_count = 0;
    let mut i = 0;

    while i < data.len() {
        let mut has_continuation = false;
        for _ in 0..5 {
            if i >= data.len() {
                break;
            }
            let b = data[i];
            i += 1;
            if b & 0x80 != 0 {
                has_continuation = true;
            } else {
                varint_count += 1;
                break;
            }
        }
        if !has_continuation && i < data.len() {
            break;
        }
    }

    varint_count >= 2
}

// ---------------------------------------------------------------------------
// Structure inference
// ---------------------------------------------------------------------------

/// Infer field boundaries from data patterns.
pub fn infer_structure(data: &[u8]) -> Vec<FieldCandidate> {
    let mut fields = Vec::new();
    let n = data.len();

    if n == 0 {
        return fields;
    }

    // Check for common header patterns
    if n >= 2 {
        let header = u16::from_be_bytes([data[0], data[1]]);
        fields.push(FieldCandidate {
            offset: 0,
            length: 2,
            field_type: "header".to_string(),
            confidence: 0.5,
            value_hex: format!("{:04X}", header),
            value_dec: Some(header as u64),
        });
    }

    // Check for length field after header
    if n >= 3 {
        let potential_len = data[2] as usize;
        if potential_len > 0 && potential_len <= n - 3 {
            fields.push(FieldCandidate {
                offset: 2,
                length: 1,
                field_type: "length".to_string(),
                confidence: 0.4,
                value_hex: format!("{:02X}", data[2]),
                value_dec: Some(data[2] as u64),
            });
        }
    }

    // Check for checksum at end (XOR or sum)
    if n >= 4 {
        let last = data[n - 1];
        let xor_checksum: u8 = data[..n - 1].iter().fold(0, |acc, &b| acc ^ b);
        let sum_checksum: u8 = data[..n - 1].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

        if last == xor_checksum {
            fields.push(FieldCandidate {
                offset: n - 1,
                length: 1,
                field_type: "checksum".to_string(),
                confidence: 0.7,
                value_hex: format!("{:02X}", last),
                value_dec: Some(last as u64),
            });
        } else if last == sum_checksum {
            fields.push(FieldCandidate {
                offset: n - 1,
                length: 1,
                field_type: "checksum".to_string(),
                confidence: 0.6,
                value_hex: format!("{:02X}", last),
                value_dec: Some(last as u64),
            });
        }
    }

    fields
}

// ---------------------------------------------------------------------------
// Main analysis function
// ---------------------------------------------------------------------------

/// Perform complete reverse engineering analysis on raw data.
pub fn analyze(data: &[u8]) -> AnalysisResult {
    let entropy = calculate_entropy(data);
    let unique_bytes = {
        let mut seen = std::collections::HashSet::new();
        data.iter().for_each(|&b| { seen.insert(b); });
        seen.len()
    };

    let patterns = find_patterns(data, 2, 16);
    let matched_protocols = fingerprint_protocols(data);
    let inferred_structure = infer_structure(data);

    let hex_preview = data.iter()
        .take(64)
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");

    let ascii_preview = data.iter()
        .take(64)
        .map(|&b| if b >= 32 && b < 127 { b as char } else { '.' })
        .collect::<String>();

    AnalysisResult {
        entropy,
        total_bytes: data.len(),
        unique_bytes,
        patterns,
        matched_protocols,
        inferred_structure,
        hex_preview,
        ascii_preview,
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn reverse_engineer_analyze(hex_data: String) -> Result<AnalysisResult, String> {
    // Decode hex string to bytes
    let bytes = decode_hex(&hex_data)
        .map_err(|e| format!("Hex decode error: {}", e))?;

    Ok(analyze(&bytes))
}

#[tauri::command]
pub fn reverse_engineer_analyze_file(path: String) -> Result<AnalysisResult, String> {
    use std::fs;
    let bytes = fs::read(&path)
        .map_err(|e| format!("File read error: {}", e))?;

    Ok(analyze(&bytes))
}

fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if s.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }

    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|e| format!("Invalid hex: {}", e))
}
