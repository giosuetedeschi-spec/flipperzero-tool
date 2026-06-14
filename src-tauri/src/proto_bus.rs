//! ProtoBus - High-level Protobuf communication layer for Flipper Zero.
//!
//! Provides:
//!   - Typed message structs matching flipper.proto
//!   - Varint encoding/decoding (protobuf wire format)
//!   - Message framing (length-prefixed protobuf)
//!   - RPC request/response matching via sequence_id
//!   - Session management (ping, stop, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::errors::AppError;
use super::serial::{FlipperConnection, write_raw, read_raw};

// ---------------------------------------------------------------------------
// Message types (matching flipper.proto)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FileInfoProto {
    pub name: String,
    pub size: u32,
    pub is_dir: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcContent {
    StorageListRequest { path: String },
    StorageListResponse { files: Vec<FileInfoProto> },
    StorageReadRequest { path: String },
    StorageReadResponse { data: Vec<u8> },
    StorageWriteRequest { path: String, data: Vec<u8> },
    StorageWriteResponse,
    StorageDeleteRequest { path: String },
    StorageDeleteResponse,
    StorageMkdirRequest { path: String },
    StorageMkdirResponse,
    DeviceInfoRequest,
    DeviceInfoResponse { name: String, model: String, firmware: String },
    PingRequest { data: Vec<u8> },
    PingResponse { data: Vec<u8> },
    StopSession,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcMessage {
    pub sequence_id: u32,
    pub content: Option<RpcContent>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionState {
    pub session_id: u32,
    pub next_sequence: u32,
    pub pending_requests: HashMap<u32, std::sync::mpsc::Sender<RpcMessage>>,
    pub connected: bool,
}

impl Default for SessionState {
    fn default() -> Self {
        SessionState {
            session_id: 1,
            next_sequence: 1,
            pending_requests: HashMap::new(),
            connected: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Varint encoding (protobuf wire format)
// ---------------------------------------------------------------------------

/// Encode a value as a protobuf varint (LEB128).
pub fn encode_varint(value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    let mut val = value;
    loop {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            byte |= 0x80;
        }
        result.push(byte);
        if val == 0 {
            break;
        }
    }
    result
}

/// Decode a varint from bytes. Returns (value, bytes_consumed).
pub fn decode_varint(data: &[u8]) -> Result<(u64, usize), AppError> {
    let mut value: u64 = 0;
    let mut shift = 0;
    for (i, &byte) in data.iter().enumerate() {
        if i >= 10 {
            return Err(AppError::ParseError("Varint too long".to_string()));
        }
        value |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok((value, i + 1));
        }
        shift += 7;
    }
    Err(AppError::ParseError("Incomplete varint".to_string()))
}

// ---------------------------------------------------------------------------
// Field encoding helpers
// ---------------------------------------------------------------------------

/// Encode a field tag + wire type.
fn encode_tag(field_number: u32, wire_type: u64) -> Vec<u8> {
    encode_varint((field_number as u64) << 3 | wire_type)
}

/// Encode a string field (field number, string).
fn encode_string_field(field: u32, s: &str) -> Vec<u8> {
    let mut result = encode_tag(field, 2);
    result.extend_from_slice(&encode_varint(s.len() as u64));
    result.extend_from_slice(s.as_bytes());
    result
}

/// Encode a varint field.
fn encode_varint_field(field: u32, value: u64) -> Vec<u8> {
    let mut result = encode_tag(field, 0);
    result.extend_from_slice(&encode_varint(value));
    result
}

/// Encode a bytes field.
fn encode_bytes_field(field: u32, data: &[u8]) -> Vec<u8> {
    let mut result = encode_tag(field, 2);
    result.extend_from_slice(&encode_varint(data.len() as u64));
    result.extend_from_slice(data);
    result
}

/// Encode a message as length-delimited bytes.
fn encode_message_field(field: u32, inner: &[u8]) -> Vec<u8> {
    encode_bytes_field(field, inner)
}

// ---------------------------------------------------------------------------
// RPC message encoding/decoding
// ---------------------------------------------------------------------------

impl RpcMessage {
    /// Serialize to protobuf wire format bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Field 1: sequence_id (varint)
        if self.sequence_id != 0 {
            result.extend_from_slice(&encode_varint_field(1, self.sequence_id as u64));
        }

        // Oneof content
        if let Some(ref content) = self.content {
            match content {
                RpcContent::StorageListRequest { path } => {
                    // Field 10: storage_list (message)
                    let inner = encode_string_field(1, path);
                    result.extend_from_slice(&encode_message_field(10, &inner));
                }
                RpcContent::StorageListResponse { files } => {
                    // Field 11: storage_list_response
                    let mut inner = Vec::new();
                    for file in files {
                        let mut file_msg = Vec::new();
                        file_msg.extend_from_slice(&encode_string_field(1, &file.name));
                        if file.size != 0 {
                            file_msg.extend_from_slice(&encode_varint_field(2, file.size as u64));
                        }
                        if file.is_dir {
                            file_msg.extend_from_slice(&encode_varint_field(3, 1));
                        }
                        inner.extend_from_slice(&encode_message_field(1, &file_msg));
                    }
                    result.extend_from_slice(&encode_message_field(11, &inner));
                }
                RpcContent::StorageReadRequest { path } => {
                    let inner = encode_string_field(1, path);
                    result.extend_from_slice(&encode_message_field(12, &inner));
                }
                RpcContent::StorageReadResponse { data } => {
                    let mut inner = Vec::new();
                    inner.extend_from_slice(&encode_bytes_field(1, data));
                    result.extend_from_slice(&encode_message_field(13, &inner));
                }
                RpcContent::StorageWriteRequest { path, data } => {
                    let mut inner = encode_string_field(1, path);
                    inner.extend_from_slice(&encode_bytes_field(2, data));
                    result.extend_from_slice(&encode_message_field(14, &inner));
                }
                RpcContent::StorageWriteResponse => {
                    result.extend_from_slice(&encode_message_field(15, &[]));
                }
                RpcContent::StorageDeleteRequest { path } => {
                    let inner = encode_string_field(1, path);
                    result.extend_from_slice(&encode_message_field(16, &inner));
                }
                RpcContent::StorageDeleteResponse => {
                    result.extend_from_slice(&encode_message_field(17, &[]));
                }
                RpcContent::StorageMkdirRequest { path } => {
                    let inner = encode_string_field(1, path);
                    result.extend_from_slice(&encode_message_field(18, &inner));
                }
                RpcContent::StorageMkdirResponse => {
                    result.extend_from_slice(&encode_message_field(19, &[]));
                }
                RpcContent::DeviceInfoRequest => {
                    result.extend_from_slice(&encode_message_field(20, &[]));
                }
                RpcContent::DeviceInfoResponse { name, model, firmware } => {
                    let mut inner = encode_string_field(1, name);
                    inner.extend_from_slice(&encode_string_field(2, model));
                    inner.extend_from_slice(&encode_string_field(3, firmware));
                    result.extend_from_slice(&encode_message_field(21, &inner));
                }
                RpcContent::PingRequest { data } => {
                    let inner = encode_bytes_field(1, data);
                    result.extend_from_slice(&encode_message_field(22, &inner));
                }
                RpcContent::PingResponse { data } => {
                    let inner = encode_bytes_field(1, data);
                    result.extend_from_slice(&encode_message_field(23, &inner));
                }
                RpcContent::StopSession => {
                    result.extend_from_slice(&encode_message_field(28, &[]));
                }
            }
        }

        result
    }

    /// Deserialize from protobuf wire format bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, AppError> {
        let mut sequence_id = 0u32;
        let mut content: Option<RpcContent> = None;
        let mut i = 0;

        while i < data.len() {
            let (tag, tag_len) = decode_varint(&data[i..])?;
            i += tag_len;
            let field_number = (tag >> 3) as u32;
            let wire_type = tag & 0x7;

            match wire_type {
                0 => {
                    // Varint
                    let (value, varint_len) = decode_varint(&data[i..])?;
                    i += varint_len;
                    if field_number == 1 {
                        sequence_id = value as u32;
                    }
                }
                2 => {
                    // Length-delimited (string, bytes, embedded message)
                    let (length, len_len) = decode_varint(&data[i..])?;
                    i += len_len;
                    let field_data = &data[i..i + length as usize];
                    i += length as usize;

                    content = match field_number {
                        10 => {
                            // StorageListRequest
                            let path = decode_string_field(field_data, 1)?;
                            Some(RpcContent::StorageListRequest { path })
                        }
                        11 => {
                            // StorageListResponse
                            let files = decode_file_list(field_data)?;
                            Some(RpcContent::StorageListResponse { files })
                        }
                        12 => {
                            let path = decode_string_field(field_data, 1)?;
                            Some(RpcContent::StorageReadRequest { path })
                        }
                        13 => {
                            let data_vec = decode_bytes_field(field_data, 1)?;
                            Some(RpcContent::StorageReadResponse { data: data_vec })
                        }
                        14 => {
                            let path = decode_string_field(field_data, 1)?;
                            let data_vec = decode_bytes_field(field_data, 2)?;
                            Some(RpcContent::StorageWriteRequest { path, data: data_vec })
                        }
                        15 => Some(RpcContent::StorageWriteResponse),
                        16 => {
                            let path = decode_string_field(field_data, 1)?;
                            Some(RpcContent::StorageDeleteRequest { path })
                        }
                        17 => Some(RpcContent::StorageDeleteResponse),
                        18 => {
                            let path = decode_string_field(field_data, 1)?;
                            Some(RpcContent::StorageMkdirRequest { path })
                        }
                        19 => Some(RpcContent::StorageMkdirResponse),
                        20 => Some(RpcContent::DeviceInfoRequest),
                        21 => {
                            let name = decode_string_field(field_data, 1)?;
                            let model = decode_string_field(field_data, 2)?;
                            let firmware = decode_string_field(field_data, 3)?;
                            Some(RpcContent::DeviceInfoResponse { name, model, firmware })
                        }
                        22 => {
                            let data_vec = decode_bytes_field(field_data, 1)?;
                            Some(RpcContent::PingRequest { data: data_vec })
                        }
                        23 => {
                            let data_vec = decode_bytes_field(field_data, 1)?;
                            Some(RpcContent::PingResponse { data: data_vec })
                        }
                        28 => Some(RpcContent::StopSession),
                        _ => None,
                    };
                }
                _ => return Err(AppError::ParseError(format!("Unknown wire type: {}", wire_type))),
            }
        }

        Ok(RpcMessage { sequence_id, content })
    }
}

// ---------------------------------------------------------------------------
// Field decoding helpers
// ---------------------------------------------------------------------------

fn decode_string_field(data: &[u8], field: u32) -> Result<String, AppError> {
    let mut i = 0;
    while i < data.len() {
        let (tag, tag_len) = decode_varint(&data[i..])?;
        i += tag_len;
        if (tag >> 3) as u32 == field {
            let (length, len_len) = decode_varint(&data[i..])?;
            i += len_len;
            return String::from_utf8(data[i..i + length as usize].to_vec())
                .map_err(|e| AppError::ParseError(format!("UTF-8 error: {}", e)));
        }
        // Skip unknown field
        let wire_type = tag & 0x7;
        match wire_type {
            0 => { let (_, l) = decode_varint(&data[i..])?; i += l; }
            2 => { let (l, ll) = decode_varint(&data[i..])?; i += ll + l as usize; }
            _ => break,
        }
    }
    Ok(String::new())
}

fn decode_bytes_field(data: &[u8], field: u32) -> Result<Vec<u8>, AppError> {
    let mut i = 0;
    while i < data.len() {
        let (tag, tag_len) = decode_varint(&data[i..])?;
        i += tag_len;
        if (tag >> 3) as u32 == field {
            let (length, len_len) = decode_varint(&data[i..])?;
            i += len_len;
            return Ok(data[i..i + length as usize].to_vec());
        }
        let wire_type = tag & 0x7;
        match wire_type {
            0 => { let (_, l) = decode_varint(&data[i..])?; i += l; }
            2 => { let (l, ll) = decode_varint(&data[i..])?; i += ll + l as usize; }
            _ => break,
        }
    }
    Ok(Vec::new())
}

fn decode_file_list(data: &[u8]) -> Result<Vec<FileInfoProto>, AppError> {
    let mut files = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let (tag, tag_len) = decode_varint(&data[i..])?;
        i += tag_len;
        if (tag >> 3) as u32 == 1 {
            let (length, len_len) = decode_varint(&data[i..])?;
            i += len_len;
            let file_data = &data[i..i + length as usize];
            i += length as usize;

            let name = decode_string_field(file_data, 1).unwrap_or_default();
            let mut size = 0u32;
            let mut is_dir = false;

            let mut fi = 0;
            while fi < file_data.len() {
                let (ftag, ftlen) = decode_varint(&file_data[fi..])?;
                fi += ftlen;
                if (ftag >> 3) == 2 {
                    let (val, vlen) = decode_varint(&file_data[fi..])?;
                    fi += vlen;
                    size = val as u32;
                } else if (ftag >> 3) == 3 {
                    let (val, vlen) = decode_varint(&file_data[fi..])?;
                    fi += vlen;
                    is_dir = val != 0;
                } else {
                    break;
                }
            }

            files.push(FileInfoProto { name, size, is_dir });
        } else {
            break;
        }
    }
    Ok(files)
}

// ---------------------------------------------------------------------------
// High-level RPC API
// ---------------------------------------------------------------------------


/// Send an RPC command and wait for the response.
/// NOTE: ProtoBus requires serial read/write which are on FlipperConnection.
/// For now, protobuf encode/decode is available; serial integration requires
/// extending FlipperConnection with port access.
pub fn rpc_command(
    _state: &Arc<std::sync::Mutex<super::serial::FlipperConnection>>,
    _content: RpcContent,
) -> Result<RpcMessage, AppError> {
    // TODO: integrate with serial port write/read
    // For now, return an error indicating serial ProtoBus is not yet connected
    Err(AppError::SerialError("ProtoBus serial integration pending".to_string()))
}

/// Encode an RPC message to bytes (for sending over serial manually).
pub fn encode_rpc(content: RpcContent, sequence_id: u32) -> Vec<u8> {
    RpcMessage { sequence_id, content: Some(content) }.to_bytes()
}

/// Decode an RPC message from bytes.
pub fn decode_rpc(data: &[u8]) -> Result<RpcMessage, AppError> {
    RpcMessage::from_bytes(data)
}


/// List files on the Flipper Zero.
pub fn proto_list_dir(
    conn: &Arc<Mutex<FlipperConnection>>,
    path: &str,
) -> Result<Vec<FileInfoProto>, AppError> {
    let resp = rpc_command(conn, RpcContent::StorageListRequest { path: path.to_string() })?;
    match resp.content {
        Some(RpcContent::StorageListResponse { files }) => Ok(files),
        _ => Err(AppError::SerialError("Unexpected response type".to_string())),
    }
}

/// Read a file from the Flipper Zero.
pub fn proto_read_file(
    conn: &Arc<Mutex<FlipperConnection>>,
    path: &str,
) -> Result<Vec<u8>, AppError> {
    let resp = rpc_command(conn, RpcContent::StorageReadRequest { path: path.to_string() })?;
    match resp.content {
        Some(RpcContent::StorageReadResponse { data }) => Ok(data),
        _ => Err(AppError::SerialError("Unexpected response type".to_string())),
    }
}

/// Write a file to the Flipper Zero.
pub fn proto_write_file(
    conn: &Arc<Mutex<FlipperConnection>>,
    path: &str,
    data: &[u8],
) -> Result<(), AppError> {
    let resp = rpc_command(conn, RpcContent::StorageWriteRequest {
        path: path.to_string(),
        data: data.to_vec(),
    })?;
    match resp.content {
        Some(RpcContent::StorageWriteResponse) => Ok(()),
        _ => Err(AppError::SerialError("Unexpected response type".to_string())),
    }
}

/// Create a directory on the Flipper Zero.
pub fn proto_mkdir(
    conn: &Arc<Mutex<FlipperConnection>>,
    path: &str,
) -> Result<(), AppError> {
    let resp = rpc_command(conn, RpcContent::StorageMkdirRequest { path: path.to_string() })?;
    match resp.content {
        Some(RpcContent::StorageMkdirResponse) => Ok(()),
        _ => Err(AppError::SerialError("Unexpected response type".to_string())),
    }
}

/// Delete a file/directory on the Flipper Zero.
pub fn proto_delete(
    conn: &Arc<Mutex<FlipperConnection>>,
    path: &str,
) -> Result<(), AppError> {
    let resp = rpc_command(conn, RpcContent::StorageDeleteRequest { path: path.to_string() })?;
    match resp.content {
        Some(RpcContent::StorageDeleteResponse) => Ok(()),
        _ => Err(AppError::SerialError("Unexpected response type".to_string())),
    }
}

/// Get Flipper Zero device info.
pub fn proto_device_info(
    conn: &Arc<Mutex<FlipperConnection>>,
) -> Result<(String, String, String), AppError> {
    let resp = rpc_command(conn, RpcContent::DeviceInfoRequest)?;
    match resp.content {
        Some(RpcContent::DeviceInfoResponse { name, model, firmware }) => Ok((name, model, firmware)),
        _ => Err(AppError::SerialError("Unexpected response type".to_string())),
    }
}

/// Ping the Flipper Zero.
pub fn proto_ping(
    conn: &Arc<Mutex<FlipperConnection>>,
    data: &[u8],
) -> Result<Vec<u8>, AppError> {
    let resp = rpc_command(conn, RpcContent::PingRequest { data: data.to_vec() })?;
    match resp.content {
        Some(RpcContent::PingResponse { data }) => Ok(data),
        _ => Err(AppError::SerialError("Unexpected response type".to_string())),
    }
}
