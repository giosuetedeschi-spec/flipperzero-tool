//! Flipper Zero serial connection management.

use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::Duration;
use super::errors::AppError;
use super::proto::{encode_frame, decode_frame, FLIPPER_BAUD, FLIPPER_TIMEOUT};
use serialport::{SerialPort, UsbPortInfo};

pub struct FlipperConnection {
    port: Box<dyn SerialPort>,
    read_buf: Vec<u8>,
}

/// Global connection state (thread-safe)
pub type FlipperState = std::sync::Arc<Mutex<Option<FlipperConnection>>>;

impl FlipperConnection {
    /// Open a serial connection to the Flipper.
    pub fn open(port_name: &str) -> Result<Self, AppError> {
        let port = serialport::new(port_name, FLIPPER_BAUD)
            .timeout(FLIPPER_TIMEOUT)
            .open()
            .map_err(|e| AppError::SerialError(format!("Cannot open {}: {}", port_name, e)))?;

        Ok(Self {
            port,
            read_buf: Vec::new(),
        })
    }

    /// Send a protobuf message and wait for response.
    pub fn send_recv<M: prost::Message + Default, R: prost::Message + Default>(
        &mut self,
        request: &M,
    ) -> Result<R, AppError> {
        // Encode and send
        let frame = encode_frame(request)?;
        self.port
            .write_all(&frame)
            .map_err(|e| AppError::SerialError(format!("Write error: {}", e)))?;
        self.port
            .flush()
            .map_err(|e| AppError::SerialError(format!("Flush error: {}", e)))?;

        // Read response
        decode_frame(&mut self.port, FLIPPER_TIMEOUT)
    }

    /// Send a message without waiting for response.
    pub fn send_only<M: prost::Message>(&mut self, request: &M) -> Result<(), AppError> {
        let frame = encode_frame(request)?;
        self.port
            .write_all(&frame)
            .map_err(|e| AppError::SerialError(format!("Write error: {}", e)))?;
        self.port
            .flush()
            .map_err(|e| AppError::SerialError(format!("Flush error: {}", e)))?;
        Ok(())
    }

    /// Read a single response message.
    pub fn recv<M: prost::Message + Default>(&mut self) -> Result<M, AppError> {
        decode_frame(&mut self.port, FLIPPER_TIMEOUT)
    }

    /// Close the connection (drops the port).
    pub fn close(self) {
        // Port is dropped automatically
    }
}

/// Check if a serial port is a Flipper Zero device.
pub fn is_flipper_port(info: &UsbPortInfo) -> bool {
    info.vid == super::proto::FLIPPER_VID && info.pid == super::proto::FLIPPER_PID
}
