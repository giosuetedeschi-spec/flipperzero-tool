//! Length-delimited framing over a [`Transport`].
//!
//! The Flipper's RPC stream is protobuf's standard length-delimited encoding: a
//! varint byte count followed by exactly that many bytes of message. Nothing in
//! the stream marks a boundary otherwise, so a reader that guesses will
//! desynchronise permanently after the first partial read.
//!
//! This is where the BLE case is won or lost. A frame does not arrive in one
//! piece: it dribbles in across notifications of a few dozen bytes, and the
//! length prefix itself can straddle two of them. [`FrameReader`] therefore
//! buffers and only yields whole frames.

use crate::errors::AppError;
use crate::serial::encode_varint;
use crate::transport::Transport;

/// Refuse absurd frame lengths rather than trying to allocate them.
///
/// A corrupted or desynchronised stream will decode a varint into something
/// enormous; without this guard the first such byte sequence becomes an
/// out-of-memory abort instead of a recoverable error.
const MAX_FRAME_LEN: u64 = 1024 * 1024;

/// Write one length-delimited frame, split to the transport's chunk limit.
pub fn write_frame(transport: &mut dyn Transport, payload: &[u8]) -> Result<(), AppError> {
    let mut framed = Vec::with_capacity(payload.len() + 8);
    encode_varint(&mut framed, payload.len() as u64);
    framed.extend_from_slice(payload);

    let chunk = transport.max_write_chunk().max(1);
    for part in framed.chunks(chunk) {
        transport.write(part)?;
    }
    Ok(())
}

/// Reassembles whole frames from a byte stream that arrives in arbitrary pieces.
#[derive(Debug, Default)]
pub struct FrameReader {
    buffer: Vec<u8>,
}

impl FrameReader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed freshly read bytes into the reader.
    pub fn push(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// How many bytes are buffered but not yet part of a complete frame.
    pub fn buffered(&self) -> usize {
        self.buffer.len()
    }

    /// Pull the next complete frame, if one has fully arrived.
    ///
    /// Returns `Ok(None)` when more bytes are needed -- the normal case
    /// mid-transfer, not an error.
    pub fn next_frame(&mut self) -> Result<Option<Vec<u8>>, AppError> {
        let Some((len, header_len)) = decode_varint_prefix(&self.buffer)? else {
            return Ok(None);
        };

        if len > MAX_FRAME_LEN {
            return Err(AppError::ParseError(format!(
                "Frame length {} exceeds the {} byte limit; the stream is desynchronised",
                len, MAX_FRAME_LEN
            )));
        }

        let total = header_len + len as usize;
        if self.buffer.len() < total {
            return Ok(None);
        }

        let frame = self.buffer[header_len..total].to_vec();
        self.buffer.drain(..total);
        Ok(Some(frame))
    }
}

/// Decode a varint at the start of `data`.
///
/// Returns the value and how many bytes it occupied, or `None` when the varint
/// is itself still incomplete -- which happens on BLE when a notification
/// boundary lands inside the length prefix.
fn decode_varint_prefix(data: &[u8]) -> Result<Option<(u64, usize)>, AppError> {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;

    for (i, &byte) in data.iter().enumerate() {
        value |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok(Some((value, i + 1)));
        }
        shift += 7;
        if shift >= 64 {
            return Err(AppError::ParseError(
                "Varint length prefix overflows 64 bits".to_string(),
            ));
        }
    }

    Ok(None)
}

/// Read from `transport` until `reader` can yield a frame, or the budget runs out.
///
/// `max_idle_polls` bounds how many consecutive empty reads are tolerated, so a
/// silent device produces a timeout instead of an infinite loop.
pub fn read_frame_blocking(
    transport: &mut dyn Transport,
    reader: &mut FrameReader,
    max_idle_polls: usize,
) -> Result<Vec<u8>, AppError> {
    let mut scratch = [0u8; 512];
    let mut idle = 0usize;

    loop {
        if let Some(frame) = reader.next_frame()? {
            return Ok(frame);
        }

        let n = transport.read(&mut scratch)?;
        if n == 0 {
            idle += 1;
            if idle > max_idle_polls {
                return Err(AppError::SerialError(
                    "Timed out waiting for a complete RPC frame".to_string(),
                ));
            }
            continue;
        }
        idle = 0;
        reader.push(&scratch[..n]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::loopback::LoopbackTransport;

    fn frame_bytes(payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        encode_varint(&mut out, payload.len() as u64);
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn writes_length_prefixed_frame() {
        let (mut transport, buffers) = LoopbackTransport::new();
        write_frame(&mut transport, b"hello").unwrap();

        let written = buffers.lock().unwrap().to_device.clone();
        assert_eq!(written, frame_bytes(b"hello"));
    }

    #[test]
    fn reads_back_what_was_written() {
        let mut reader = FrameReader::new();
        reader.push(&frame_bytes(b"payload"));
        assert_eq!(reader.next_frame().unwrap().unwrap(), b"payload");
        assert!(reader.next_frame().unwrap().is_none());
    }

    #[test]
    fn reassembles_a_frame_split_byte_by_byte() {
        // The BLE case: every byte arrives in its own notification, including the
        // bytes of the length prefix.
        let payload = vec![7u8; 300]; // long enough to need a two-byte varint
        let bytes = frame_bytes(&payload);

        let mut reader = FrameReader::new();
        for (i, byte) in bytes.iter().enumerate() {
            reader.push(&[*byte]);
            if i + 1 < bytes.len() {
                assert!(
                    reader.next_frame().unwrap().is_none(),
                    "yielded a frame after only {} of {} bytes",
                    i + 1,
                    bytes.len()
                );
            }
        }
        assert_eq!(reader.next_frame().unwrap().unwrap(), payload);
    }

    #[test]
    fn separates_frames_that_arrive_coalesced() {
        let mut reader = FrameReader::new();
        let mut stream = frame_bytes(b"one");
        stream.extend_from_slice(&frame_bytes(b"two"));
        stream.extend_from_slice(&frame_bytes(b"three"));
        reader.push(&stream);

        assert_eq!(reader.next_frame().unwrap().unwrap(), b"one");
        assert_eq!(reader.next_frame().unwrap().unwrap(), b"two");
        assert_eq!(reader.next_frame().unwrap().unwrap(), b"three");
        assert!(reader.next_frame().unwrap().is_none());
        assert_eq!(reader.buffered(), 0);
    }

    #[test]
    fn write_respects_a_small_mtu() {
        // 20 bytes is a realistic BLE payload size before MTU negotiation.
        let (mut transport, buffers) = LoopbackTransport::with_limits(20, usize::MAX);
        let payload = vec![0xABu8; 100];
        write_frame(&mut transport, &payload).unwrap();

        assert_eq!(buffers.lock().unwrap().to_device, frame_bytes(&payload));
    }

    #[test]
    fn round_trips_through_a_dribbling_transport() {
        let (mut transport, buffers) = LoopbackTransport::with_limits(20, 7);
        buffers
            .lock()
            .unwrap()
            .from_device
            .extend(frame_bytes(b"a reply that spans several reads"));

        let mut reader = FrameReader::new();
        let frame = read_frame_blocking(&mut transport, &mut reader, 16).unwrap();
        assert_eq!(frame, b"a reply that spans several reads");
    }

    #[test]
    fn silent_transport_times_out_instead_of_hanging() {
        let (mut transport, _buffers) = LoopbackTransport::new();
        let mut reader = FrameReader::new();
        let err = read_frame_blocking(&mut transport, &mut reader, 3).unwrap_err();
        assert!(matches!(err, AppError::SerialError(_)), "got {:?}", err);
    }

    #[test]
    fn rejects_an_absurd_frame_length() {
        let mut reader = FrameReader::new();
        let mut bogus = Vec::new();
        encode_varint(&mut bogus, MAX_FRAME_LEN + 1);
        reader.push(&bogus);
        assert!(reader.next_frame().is_err());
    }

    #[test]
    fn incomplete_varint_prefix_is_not_an_error() {
        // 0x80 has the continuation bit set, so the varint is unfinished.
        let mut reader = FrameReader::new();
        reader.push(&[0x80]);
        assert!(reader.next_frame().unwrap().is_none());
    }

    #[test]
    fn empty_frame_round_trips() {
        let mut reader = FrameReader::new();
        reader.push(&frame_bytes(b""));
        assert_eq!(reader.next_frame().unwrap().unwrap(), Vec::<u8>::new());
    }
}
