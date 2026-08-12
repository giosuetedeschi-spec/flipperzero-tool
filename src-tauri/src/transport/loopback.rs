//! An in-memory [`Transport`] used to test framing and sessions without hardware.
//!
//! Two independent queues stand in for the two directions of the link, so a test
//! can push bytes the "device" would send and inspect the bytes the client wrote.
//! A configurable [`max_write_chunk`](Transport::max_write_chunk) lets tests
//! reproduce the small-MTU behaviour of BLE, which is where chunking bugs live.

use super::{Transport, TransportKind};
use crate::errors::AppError;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Shared queues behind a [`LoopbackTransport`], so a test can drive both ends.
#[derive(Debug, Default)]
pub struct LoopbackBuffers {
    /// Bytes the client has written (i.e. what the device would receive).
    pub to_device: Vec<u8>,
    /// Bytes queued for the client to read (i.e. what the device sent).
    pub from_device: VecDeque<u8>,
}

pub struct LoopbackTransport {
    buffers: Arc<Mutex<LoopbackBuffers>>,
    max_chunk: usize,
    /// Bytes handed out per `read` call. Small values reproduce a link that
    /// dribbles a frame across several reads.
    read_granularity: usize,
}

impl LoopbackTransport {
    /// A loopback with no chunking limits -- the USB-like case.
    pub fn new() -> (Self, Arc<Mutex<LoopbackBuffers>>) {
        Self::with_limits(usize::MAX, usize::MAX)
    }

    /// A loopback that writes at most `max_chunk` bytes at a time and returns at
    /// most `read_granularity` bytes per read -- the BLE-like case.
    pub fn with_limits(
        max_chunk: usize,
        read_granularity: usize,
    ) -> (Self, Arc<Mutex<LoopbackBuffers>>) {
        let buffers = Arc::new(Mutex::new(LoopbackBuffers::default()));
        let transport = Self {
            buffers: Arc::clone(&buffers),
            max_chunk,
            read_granularity: read_granularity.max(1),
        };
        (transport, buffers)
    }
}

impl Transport for LoopbackTransport {
    fn kind(&self) -> TransportKind {
        TransportKind::Loopback
    }

    fn write(&mut self, data: &[u8]) -> Result<(), AppError> {
        let mut buffers = self
            .buffers
            .lock()
            .map_err(|_| AppError::SerialError("Loopback buffers poisoned".into()))?;
        // Record the split the way the real link would perform it, so a test can
        // assert that framing never hands over an oversized chunk.
        for chunk in data.chunks(self.max_chunk.max(1)) {
            buffers.to_device.extend_from_slice(chunk);
        }
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, AppError> {
        let mut buffers = self
            .buffers
            .lock()
            .map_err(|_| AppError::SerialError("Loopback buffers poisoned".into()))?;
        let n = buf
            .len()
            .min(self.read_granularity)
            .min(buffers.from_device.len());
        for slot in buf.iter_mut().take(n) {
            *slot = buffers
                .from_device
                .pop_front()
                .expect("length was just checked");
        }
        Ok(n)
    }

    fn max_write_chunk(&self) -> usize {
        self.max_chunk
    }
}
