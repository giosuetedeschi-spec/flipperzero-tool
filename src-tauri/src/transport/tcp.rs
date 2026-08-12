//! TCP transport, used to reach the mock Flipper.
//!
//! The mock speaks exactly the wire format a real device does -- length-delimited
//! protobuf RPC -- so the whole stack above [`Transport`] runs unchanged against
//! it. That is the point: without hardware, this is the only way to exercise
//! sessions, framing and commands end to end.
//!
//! It is not a production link. Nothing ships a Flipper reachable over TCP.

use super::{Transport, TransportKind};
use crate::errors::AppError;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Matches the real transports: long enough to ride out a slow reply, short
/// enough that the framing layer's idle polling stays responsive.
const READ_TIMEOUT: Duration = Duration::from_millis(100);

pub struct TcpTransport {
    stream: TcpStream,
}

impl TcpTransport {
    pub fn connect<A: ToSocketAddrs>(address: A) -> Result<Self, AppError> {
        let stream = TcpStream::connect(address)
            .map_err(|e| AppError::SerialError(format!("Cannot reach the mock Flipper: {}", e)))?;
        stream
            .set_read_timeout(Some(READ_TIMEOUT))
            .map_err(|e| AppError::SerialError(format!("Cannot set read timeout: {}", e)))?;
        // Frames are small and latency matters more than packing here; without
        // this a reply can sit in the kernel waiting for more data that the
        // request/response pattern will never produce.
        stream
            .set_nodelay(true)
            .map_err(|e| AppError::SerialError(format!("Cannot disable Nagle: {}", e)))?;
        Ok(Self { stream })
    }
}

impl Transport for TcpTransport {
    fn kind(&self) -> TransportKind {
        TransportKind::Loopback
    }

    fn write(&mut self, data: &[u8]) -> Result<(), AppError> {
        self.stream
            .write_all(data)
            .map_err(|e| AppError::SerialError(format!("Write error: {}", e)))?;
        self.stream
            .flush()
            .map_err(|e| AppError::SerialError(format!("Flush error: {}", e)))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, AppError> {
        match self.stream.read(buf) {
            Ok(n) => Ok(n),
            // A timeout means "nothing yet", which the framing layer polls
            // through; reporting it as an error would abort every idle moment.
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                Ok(0)
            }
            Err(e) => Err(AppError::SerialError(format!("Read error: {}", e))),
        }
    }

    fn close(&mut self) -> Result<(), AppError> {
        // Best effort: the peer may already be gone, which is not a failure.
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
        Ok(())
    }
}
