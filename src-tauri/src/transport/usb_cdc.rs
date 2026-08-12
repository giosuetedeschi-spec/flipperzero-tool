//! USB CDC serial transport -- the desktop link to the Flipper.
//!
//! Unlike [`crate::serial`], which re-opened the port for every command, this
//! holds the port open for the life of the transport. That is a requirement,
//! not a preference: an RPC session is stateful, and reopening the port
//! restarts the device's CLI and discards any session in progress.

use super::{Transport, TransportKind};
use crate::errors::AppError;
use crate::serial::{FLIPPER_BAUD, FLIPPER_TIMEOUT, find_flipper};
use std::io::{Read, Write};
use std::time::{Duration, Instant};

/// The CLI command that hands the serial stream over to the protobuf RPC layer.
///
/// A freshly opened port speaks the interactive text CLI. Until this is sent and
/// acknowledged, framed protobuf written to the port is interpreted as console
/// input and silently discarded.
const START_RPC_SESSION: &str = "start_rpc_session";

/// The prompt the CLI prints when it is ready for a command.
const CLI_PROMPT: &str = ">:";

pub struct UsbCdcTransport {
    port: Box<dyn serialport::SerialPort>,
    port_name: String,
}

impl UsbCdcTransport {
    /// Open a port by name, without entering RPC mode.
    pub fn open(port_name: &str) -> Result<Self, AppError> {
        if port_name.is_empty() {
            return Err(AppError::SerialError(
                "Port name cannot be empty".to_string(),
            ));
        }
        let port = serialport::new(port_name, FLIPPER_BAUD)
            .timeout(FLIPPER_TIMEOUT)
            .open()
            .map_err(|e| AppError::SerialError(format!("Cannot open {}: {}", port_name, e)))?;
        Ok(Self {
            port,
            port_name: port_name.to_string(),
        })
    }

    /// Find a Flipper by USB VID:PID and open it.
    pub fn autodetect() -> Result<Self, AppError> {
        let port = find_flipper()?.ok_or_else(|| {
            AppError::SerialError("Flipper Zero not found (VID:PID 0483:5740)".to_string())
        })?;
        Self::open(&port.port_name)
    }

    pub fn port_name(&self) -> &str {
        &self.port_name
    }

    /// Switch the stream from the text CLI into protobuf RPC mode.
    ///
    /// Drains whatever banner or partial line is already buffered first,
    /// otherwise the leftover text is read back as if it were the first RPC
    /// frame and the stream never recovers.
    pub fn start_rpc_session(&mut self) -> Result<(), AppError> {
        self.drain_pending()?;

        self.port
            .write_all(format!("{}\r\n", START_RPC_SESSION).as_bytes())
            .map_err(|e| AppError::SerialError(format!("Write error: {}", e)))?;
        self.port
            .flush()
            .map_err(|e| AppError::SerialError(format!("Flush error: {}", e)))?;

        // The device echoes the command and a final prompt before switching
        // modes. Consume through that prompt so the first byte the RPC layer
        // sees belongs to a frame.
        let deadline = Instant::now() + FLIPPER_TIMEOUT;
        let mut echoed = String::new();
        let mut buf = [0u8; 256];
        while Instant::now() < deadline {
            match self.port.read(&mut buf) {
                Ok(0) => std::thread::sleep(Duration::from_millis(10)),
                Ok(n) => {
                    echoed.push_str(&String::from_utf8_lossy(&buf[..n]));
                    // Once the echoed command and its prompt have gone by, the
                    // device is in RPC mode.
                    if echoed.contains(START_RPC_SESSION) && echoed.contains(CLI_PROMPT) {
                        return Ok(());
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(e) => return Err(AppError::SerialError(format!("Read error: {}", e))),
            }
        }

        Err(AppError::SerialError(
            "Device did not acknowledge start_rpc_session".to_string(),
        ))
    }

    /// Discard anything already sitting in the receive buffer.
    fn drain_pending(&mut self) -> Result<(), AppError> {
        let mut buf = [0u8; 256];
        loop {
            match self.port.read(&mut buf) {
                Ok(0) => return Ok(()),
                Ok(_) => continue,
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => return Ok(()),
                Err(e) => return Err(AppError::SerialError(format!("Drain error: {}", e))),
            }
        }
    }
}

impl Transport for UsbCdcTransport {
    fn kind(&self) -> TransportKind {
        TransportKind::UsbCdc
    }

    fn write(&mut self, data: &[u8]) -> Result<(), AppError> {
        self.port
            .write_all(data)
            .map_err(|e| AppError::SerialError(format!("Write error: {}", e)))?;
        self.port
            .flush()
            .map_err(|e| AppError::SerialError(format!("Flush error: {}", e)))
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, AppError> {
        match self.port.read(buf) {
            Ok(n) => Ok(n),
            // A read timeout means "nothing yet", which the framing layer polls
            // through. Reporting it as an error would abort every idle moment.
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(0),
            Err(e) => Err(AppError::SerialError(format!("Read error: {}", e))),
        }
    }

    fn close(&mut self) -> Result<(), AppError> {
        // Dropping the boxed port closes the handle; nothing else to release.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `unwrap_err` would require the success type to be Debug; an open port is
    // not meaningfully printable, so match on the error instead.
    #[test]
    fn open_rejects_an_empty_port_name() {
        assert!(matches!(
            UsbCdcTransport::open("").err(),
            Some(AppError::SerialError(_))
        ));
    }

    #[test]
    fn open_reports_a_missing_device_rather_than_panicking() {
        assert!(matches!(
            UsbCdcTransport::open("/dev/definitely-not-a-real-port").err(),
            Some(AppError::SerialError(_))
        ));
    }
}
