//! Protobuf framing for Flipper Zero serial protocol.
//!
//! Each message on the wire is:
//!   [varint length] [protobuf message bytes]
//!
//! The varint is a standard protobuf variable-length integer (LEB128).

use bytes::{Buf, BufMut};
use prost::Message;
use std::io::{Read, Write};
use std::time::Duration;
use super::errors::AppError;

/// Encode a protobuf message into a length-prefixed frame.
pub fn encode_frame<M: Message>(msg: &M) -> Result<Vec<u8>, AppError> {
    let msg_bytes = msg.encode_to_vec();
    let len = msg_bytes.len();

    // Encode varint length prefix
    let mut buf = Vec::with_capacity(varint_size(len) + len);
    encode_varint(&mut buf, len as u64);
    buf.extend_from_slice(&msg_bytes);
    Ok(buf)
}

/// Decode a length-prefixed protobuf message from a reader.
pub fn decode_frame<R: Read, M: Message + Default>(
    reader: &mut R,
    timeout: Duration,
) -> Result<M, AppError> {
    // Read varint length
    let len = read_varint(reader, timeout)?;

    // Read message bytes
    let mut buf = vec![0u8; len as usize];
    reader
        .read_exact(&mut buf)
        .map_err(|e| AppError::SerialError(format!("Read error: {}", e)))?;

    M::decode(&buf[..])
        .map_err(|e| AppError::SerialError(format!("Decode error: {}", e)))
}

/// Encode a varint (LEB128).
fn encode_varint(buf: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        buf.push((value as u8) | 0x80);
        value >>= 7;
    }
    buf.push(value as u8);
}

/// Read a varint from a reader.
fn read_varint<R: Read>(reader: &mut R, timeout: Duration) -> Result<u64, AppError> {
    let start = std::time::Instant::now();
    let mut result: u64 = 0;
    let mut shift: u32 = 0;

    loop {
        if start.elapsed() > timeout {
            return Err(AppError::SerialError("Varint read timeout".to_string()));
        }

        let mut byte = [0u8; 1];
        match reader.read_exact(&mut byte) {
            Ok(()) => {
                result |= (byte[0] as u64 & 0x7F) << shift;
                if byte[0] & 0x80 == 0 {
                    return Ok(result);
                }
                shift += 7;
                if shift >= 64 {
                    return Err(AppError::SerialError("Varint too long".to_string()));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                return Err(AppError::SerialError("Timeout reading varint".to_string()));
            }
            Err(e) => return Err(AppError::SerialError(format!("Read error: {}", e))),
        }
    }
}

/// Calculate the size of a varint encoding.
fn varint_size(value: usize) -> usize {
    let mut size = 1;
    let mut v = value;
    while v >= 0x80 {
        size += 1;
        v >>= 7;
    }
    size
}

/// Flipper Zero USB serial VID:PID
pub const FLIPPER_VID: u16 = 0x0483;
pub const FLIPPER_PID: u16 = 0x5740;
pub const FLIPPER_BAUD: u32 = 115200;
pub const FLIPPER_TIMEOUT: Duration = Duration::from_secs(3);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_encode_decode() {
        let values = [0u64, 1, 127, 128, 255, 16383, 16384, u64::MAX];
        for &val in &values {
            let mut buf = Vec::new();
            encode_varint(&mut buf, val);

            let mut reader = &buf[..];
            let decoded = read_varint(&mut reader, Duration::from_secs(1)).unwrap();
            assert_eq!(val, decoded, "varint roundtrip failed for {}", val);
        }
    }

    #[test]
    fn test_varint_size() {
        assert_eq!(varint_size(0), 1);
        assert_eq!(varint_size(127), 1);
        assert_eq!(varint_size(128), 2);
        assert_eq!(varint_size(16383), 2);
        assert_eq!(varint_size(16384), 3);
    }
}
