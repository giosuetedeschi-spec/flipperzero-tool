//! A session that owns its connection to the Flipper for as long as it lives.
//!
//! This replaces the pattern in [`crate::serial`], where `FlipperConnection`
//! stored only a port *name* and re-opened the port for every single command.
//! That worked well enough for one-shot CLI calls over USB, but it cannot work
//! at all for BLE: a link that takes seconds to establish and negotiate cannot
//! be torn down and rebuilt between operations, and an RPC stream is stateful --
//! sequence ids and chunked replies only make sense within one continuous
//! connection.

use super::framing::{FrameReader, read_frame_blocking, write_frame};
use crate::errors::AppError;
use crate::transport::{Transport, TransportKind};

/// How many consecutive empty reads to tolerate before declaring a timeout.
///
/// Each poll costs whatever the transport's own read timeout is, so this is a
/// multiplier on that, not a wall-clock value.
const DEFAULT_MAX_IDLE_POLLS: usize = 64;

pub struct FlipperSession {
    transport: Box<dyn Transport>,
    reader: FrameReader,
    next_command_id: u32,
    max_idle_polls: usize,
}

impl FlipperSession {
    pub fn new(transport: Box<dyn Transport>) -> Self {
        Self {
            transport,
            reader: FrameReader::new(),
            // The device treats 0 as "no command id", so ids start at 1.
            next_command_id: 1,
            max_idle_polls: DEFAULT_MAX_IDLE_POLLS,
        }
    }

    pub fn with_max_idle_polls(mut self, polls: usize) -> Self {
        self.max_idle_polls = polls;
        self
    }

    pub fn kind(&self) -> TransportKind {
        self.transport.kind()
    }

    /// Allocate the next command id, wrapping back to 1 rather than to 0.
    pub fn allocate_command_id(&mut self) -> u32 {
        let id = self.next_command_id;
        self.next_command_id = self.next_command_id.checked_add(1).unwrap_or(1);
        id
    }

    /// Send one framed message without waiting for a reply.
    pub fn send(&mut self, payload: &[u8]) -> Result<(), AppError> {
        write_frame(self.transport.as_mut(), payload)
    }

    /// Block until the next framed message arrives.
    pub fn receive(&mut self) -> Result<Vec<u8>, AppError> {
        read_frame_blocking(
            self.transport.as_mut(),
            &mut self.reader,
            self.max_idle_polls,
        )
    }

    /// Send a message and wait for the next one to come back.
    pub fn request(&mut self, payload: &[u8]) -> Result<Vec<u8>, AppError> {
        self.send(payload)?;
        self.receive()
    }

    pub fn close(&mut self) -> Result<(), AppError> {
        self.transport.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serial::encode_varint;
    use crate::transport::loopback::LoopbackTransport;

    fn framed(payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        encode_varint(&mut out, payload.len() as u64);
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn request_writes_a_frame_and_returns_the_reply() {
        let (transport, buffers) = LoopbackTransport::new();
        buffers.lock().unwrap().from_device.extend(framed(b"pong"));

        let mut session = FlipperSession::new(Box::new(transport));
        let reply = session.request(b"ping").unwrap();

        assert_eq!(reply, b"pong");
        assert_eq!(buffers.lock().unwrap().to_device, framed(b"ping"));
    }

    #[test]
    fn command_ids_are_unique_and_never_zero() {
        let (transport, _buffers) = LoopbackTransport::new();
        let mut session = FlipperSession::new(Box::new(transport));

        let ids: Vec<u32> = (0..5).map(|_| session.allocate_command_id()).collect();
        assert_eq!(ids, vec![1, 2, 3, 4, 5]);
        assert!(!ids.contains(&0), "0 means 'no command id' to the device");
    }

    #[test]
    fn command_ids_wrap_to_one_rather_than_zero() {
        let (transport, _buffers) = LoopbackTransport::new();
        let mut session = FlipperSession::new(Box::new(transport));
        session.next_command_id = u32::MAX;

        assert_eq!(session.allocate_command_id(), u32::MAX);
        assert_eq!(session.allocate_command_id(), 1);
    }

    #[test]
    fn survives_a_reply_dribbled_across_many_reads() {
        // The BLE shape: 20-byte writes, 7 bytes returned per read.
        let (transport, buffers) = LoopbackTransport::with_limits(20, 7);
        let reply = vec![0x5Au8; 250];
        buffers.lock().unwrap().from_device.extend(framed(&reply));

        let mut session = FlipperSession::new(Box::new(transport));
        assert_eq!(session.request(b"give me a long answer").unwrap(), reply);
    }

    #[test]
    fn consecutive_requests_stay_in_sync() {
        let (transport, buffers) = LoopbackTransport::new();
        {
            let mut b = buffers.lock().unwrap();
            b.from_device.extend(framed(b"first"));
            b.from_device.extend(framed(b"second"));
        }

        let mut session = FlipperSession::new(Box::new(transport));
        assert_eq!(session.request(b"a").unwrap(), b"first");
        assert_eq!(session.request(b"b").unwrap(), b"second");
    }

    #[test]
    fn silent_device_times_out() {
        let (transport, _buffers) = LoopbackTransport::new();
        let mut session = FlipperSession::new(Box::new(transport)).with_max_idle_polls(2);
        assert!(session.request(b"anyone there?").is_err());
    }
}
