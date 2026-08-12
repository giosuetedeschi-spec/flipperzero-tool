//! Byte-stream transports to the Flipper Zero.
//!
//! The device is reachable over several physically different links -- USB CDC on
//! desktop, BLE on phones, USB-OTG on Android -- but every one of them carries
//! the same length-delimited protobuf RPC stream on top. [`Transport`] is the
//! seam: everything above it (framing, sessions, commands) is written once and
//! runs unchanged on every platform.
//!
//! The trait is deliberately blocking and object-safe. Blocking matches how the
//! underlying links actually behave (`serialport` reads block; BLE notifications
//! arrive on a callback thread and are bridged through a channel), and
//! object-safety lets the session pick a transport at runtime without dragging
//! in an async-trait dependency.

use super::errors::AppError;
use serde::{Deserialize, Serialize};

pub mod ble;
pub mod loopback;
pub mod tcp;
#[cfg(desktop)]
pub mod usb_cdc;

/// Which physical link a [`Transport`] is running over.
///
/// Carried into the UI so it can explain what is and is not possible on the
/// current connection -- scanning surrounding BLE devices, for instance, cannot
/// work while the phone itself occupies the Flipper's BLE radio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportKind {
    /// USB CDC serial, desktop only.
    UsbCdc,
    /// Bluetooth Low Energy, the only link available on iOS.
    Ble,
    /// USB On-The-Go, Android only.
    UsbOtg,
    /// In-memory link used by tests and the mock device.
    Loopback,
}

impl TransportKind {
    /// Whether the Flipper's own BLE radio is occupied by this link.
    ///
    /// When it is, the device cannot also scan for surrounding BLE peripherals,
    /// and the UI must say so rather than showing an empty result as if nothing
    /// were nearby.
    pub fn occupies_device_ble_radio(self) -> bool {
        matches!(self, TransportKind::Ble)
    }
}

/// A bidirectional byte stream to the device.
///
/// Implementors only move bytes; they know nothing about framing or protobuf.
pub trait Transport: Send {
    fn kind(&self) -> TransportKind;

    /// Write the whole buffer. Implementors must respect
    /// [`max_write_chunk`](Transport::max_write_chunk) internally if the link
    /// needs it; callers pass whole frames.
    fn write(&mut self, data: &[u8]) -> Result<(), AppError>;

    /// Read whatever is available into `buf`, blocking until at least one byte
    /// arrives or the transport's own timeout elapses.
    ///
    /// Returning `Ok(0)` means "nothing yet, ask again" -- not end of stream.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, AppError>;

    /// Largest payload the link will accept in a single write.
    ///
    /// BLE negotiates an MTU in the tens of bytes, so frames must be split;
    /// USB has no meaningful limit. Framing uses this to chunk.
    fn max_write_chunk(&self) -> usize {
        usize::MAX
    }

    fn close(&mut self) -> Result<(), AppError> {
        Ok(())
    }
}
