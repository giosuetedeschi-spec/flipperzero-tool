//! BLE transport -- the only link iOS can use, and the default on Android.
//!
//! The Flipper exposes a GATT "serial service" that carries the same RPC stream
//! as USB. Three things make it harder than USB, and each is handled here:
//!
//! 1. **The MTU is tiny.** After negotiation a write is typically 20-244 bytes,
//!    so frames must be split. [`Transport::max_write_chunk`] reports the
//!    negotiated size and `rpc::framing` chunks against it.
//! 2. **There is real flow control.** The Flipper publishes how much buffer it
//!    has left. Ignoring it does not slow the transfer down -- it silently drops
//!    packets, and the RPC stream desynchronises with no error anywhere.
//! 3. **Reads are push, not pull.** Notifications arrive on a platform callback
//!    thread. [`BleLink`] bridges that into the blocking pull model the
//!    [`Transport`] trait uses.
//!
//! ## What is verified here and what is not
//!
//! The chunking, flow control and buffering logic below is exercised by tests
//! against a fake [`BleLink`]. The platform bridge underneath -- CoreBluetooth on
//! iOS, `BluetoothGatt` on Android -- cannot run in CI and is verified only on a
//! real device. The split is deliberate: everything that can be tested without
//! hardware lives on this side of the `BleLink` boundary.

use super::{Transport, TransportKind};
use crate::errors::AppError;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// GATT identifiers
// ---------------------------------------------------------------------------
//
// These come from the Flipper firmware's serial service. They are collected in
// one place because they are the single most likely thing to need correcting
// against a specific firmware build -- if the connection succeeds but no
// notification ever arrives, check these first.

/// The Flipper's serial service.
pub const SERIAL_SERVICE_UUID: &str = "8fe5b3d5-2e7f-4a98-2a48-7acc60fe0000";
/// Phone -> Flipper. Written to.
pub const RX_CHARACTERISTIC_UUID: &str = "19ed82ae-ed21-4c9d-4145-228e61fe0001";
/// Flipper -> phone. Subscribed to for notifications.
pub const TX_CHARACTERISTIC_UUID: &str = "19ed82ae-ed21-4c9d-4145-228e61fe0002";
/// Remaining device-side buffer, published by the Flipper.
pub const FLOW_CONTROL_CHARACTERISTIC_UUID: &str = "19ed82ae-ed21-4c9d-4145-228e61fe0003";

/// Conservative default before MTU negotiation completes.
///
/// The BLE 4.0 minimum ATT MTU is 23 bytes, of which 3 are protocol overhead.
/// Starting here means the first writes are small but never rejected.
pub const DEFAULT_MTU_PAYLOAD: usize = 20;

/// How long to wait for device-side buffer space before giving up on a write.
const FLOW_CONTROL_TIMEOUT: Duration = Duration::from_secs(5);

/// How long to sleep between flow-control checks while the device is congested.
const FLOW_CONTROL_POLL: Duration = Duration::from_millis(20);

// ---------------------------------------------------------------------------
// Platform bridge
// ---------------------------------------------------------------------------

/// The platform-specific half of a BLE connection.
///
/// Implemented by the mobile plugin (Swift/Kotlin via the Tauri bridge) and by
/// the fake used in tests. Kept deliberately small: it moves bytes and reports
/// two numbers, and holds no protocol knowledge at all.
pub trait BleLink: Send {
    /// Write one already-sized chunk to the RX characteristic.
    ///
    /// The caller guarantees `chunk.len() <= mtu_payload()`.
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), AppError>;

    /// Take whatever notification bytes have arrived since the last call.
    ///
    /// Returns empty when nothing has arrived; that is not an error.
    fn drain_notifications(&mut self) -> Result<Vec<u8>, AppError>;

    /// Usable payload per write, after MTU negotiation.
    fn mtu_payload(&self) -> usize;

    /// Bytes the device still has room for, from the flow-control
    /// characteristic. `None` when the device does not report it, in which case
    /// writes proceed unthrottled.
    fn free_device_buffer(&self) -> Option<usize>;

    fn is_connected(&self) -> bool;

    fn disconnect(&mut self) -> Result<(), AppError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Transport
// ---------------------------------------------------------------------------

pub struct BleTransport {
    link: Box<dyn BleLink>,
    /// Notification bytes received but not yet handed to the reader.
    inbox: VecDeque<u8>,
}

impl BleTransport {
    pub fn new(link: Box<dyn BleLink>) -> Self {
        Self {
            link,
            inbox: VecDeque::new(),
        }
    }

    /// Block until the device reports room for `wanted` bytes.
    ///
    /// Writing past the device's buffer does not fail loudly -- the packets are
    /// simply dropped, and the RPC stream then desynchronises with no error to
    /// point at. Waiting is the only way to keep the stream intact.
    fn await_buffer_space(&self, wanted: usize) -> Result<(), AppError> {
        let deadline = Instant::now() + FLOW_CONTROL_TIMEOUT;
        loop {
            match self.link.free_device_buffer() {
                // The device does not publish flow control; nothing to wait for.
                None => return Ok(()),
                Some(free) if free >= wanted => return Ok(()),
                Some(_) => {
                    if Instant::now() >= deadline {
                        return Err(AppError::SerialError(
                            "Flipper's receive buffer stayed full; the link is congested or stalled"
                                .to_string(),
                        ));
                    }
                    std::thread::sleep(FLOW_CONTROL_POLL);
                }
            }
        }
    }
}

impl Transport for BleTransport {
    fn kind(&self) -> TransportKind {
        TransportKind::Ble
    }

    fn write(&mut self, data: &[u8]) -> Result<(), AppError> {
        if !self.link.is_connected() {
            return Err(AppError::SerialError(
                "BLE link is not connected".to_string(),
            ));
        }

        // Split here as well as in the framing layer: a caller that hands over a
        // whole frame must never reach the platform bridge with an oversized
        // buffer, which most stacks reject outright.
        let chunk_size = self.link.mtu_payload().max(1);
        for chunk in data.chunks(chunk_size) {
            self.await_buffer_space(chunk.len())?;
            self.link.write_chunk(chunk)?;
        }
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, AppError> {
        if self.inbox.is_empty() {
            let received = self.link.drain_notifications()?;
            self.inbox.extend(received);
        }

        // Empty means "nothing yet", which the framing layer polls through. It
        // is not end of stream and must not be reported as an error.
        let n = buf.len().min(self.inbox.len());
        for slot in buf.iter_mut().take(n) {
            *slot = self.inbox.pop_front().expect("length was just checked");
        }
        Ok(n)
    }

    fn max_write_chunk(&self) -> usize {
        self.link.mtu_payload().max(1)
    }

    fn close(&mut self) -> Result<(), AppError> {
        self.link.disconnect()
    }
}

// ---------------------------------------------------------------------------
// Test double
// ---------------------------------------------------------------------------

/// Shared state of a [`FakeBleLink`], so a test can drive the device side.
#[derive(Debug, Default)]
pub struct FakeBleState {
    /// Every chunk the transport wrote, kept separate to assert on splitting.
    pub written_chunks: Vec<Vec<u8>>,
    /// Bytes the fake device will deliver as notifications.
    pub to_notify: VecDeque<u8>,
    /// Notification bytes released per drain, mimicking packet-sized delivery.
    pub notify_granularity: usize,
    pub mtu_payload: usize,
    pub free_buffer: Option<usize>,
    pub connected: bool,
}

/// An in-memory [`BleLink`] for testing the logic above without hardware.
pub struct FakeBleLink {
    state: Arc<Mutex<FakeBleState>>,
}

impl FakeBleLink {
    pub fn new(mtu_payload: usize) -> (Self, Arc<Mutex<FakeBleState>>) {
        let state = Arc::new(Mutex::new(FakeBleState {
            mtu_payload,
            notify_granularity: usize::MAX,
            connected: true,
            ..Default::default()
        }));
        (
            Self {
                state: Arc::clone(&state),
            },
            state,
        )
    }
}

impl BleLink for FakeBleLink {
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), AppError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AppError::SerialError("Fake BLE state poisoned".into()))?;
        if chunk.len() > state.mtu_payload {
            return Err(AppError::SerialError(format!(
                "Chunk of {} bytes exceeds the {} byte MTU payload",
                chunk.len(),
                state.mtu_payload
            )));
        }
        // Consume the device's buffer the way the real one would.
        if let Some(free) = state.free_buffer {
            state.free_buffer = Some(free.saturating_sub(chunk.len()));
        }
        state.written_chunks.push(chunk.to_vec());
        Ok(())
    }

    fn drain_notifications(&mut self) -> Result<Vec<u8>, AppError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AppError::SerialError("Fake BLE state poisoned".into()))?;
        let n = state.notify_granularity.min(state.to_notify.len());
        Ok(state.to_notify.drain(..n).collect())
    }

    fn mtu_payload(&self) -> usize {
        self.state
            .lock()
            .map(|s| s.mtu_payload)
            .unwrap_or(DEFAULT_MTU_PAYLOAD)
    }

    fn free_device_buffer(&self) -> Option<usize> {
        self.state.lock().ok().and_then(|s| s.free_buffer)
    }

    fn is_connected(&self) -> bool {
        self.state.lock().map(|s| s.connected).unwrap_or(false)
    }

    fn disconnect(&mut self) -> Result<(), AppError> {
        if let Ok(mut state) = self.state.lock() {
            state.connected = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::framing::{FrameReader, read_frame_blocking, write_frame};
    use crate::serial::encode_varint;

    fn framed(payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        encode_varint(&mut out, payload.len() as u64);
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn writes_are_split_to_the_negotiated_mtu() {
        let (link, state) = FakeBleLink::new(20);
        let mut transport = BleTransport::new(Box::new(link));

        transport.write(&vec![0xAB; 105]).unwrap();

        let chunks = &state.lock().unwrap().written_chunks;
        assert_eq!(chunks.len(), 6, "105 bytes at 20 per write");
        assert!(chunks.iter().all(|c| c.len() <= 20));
        assert_eq!(chunks.concat(), vec![0xAB; 105]);
    }

    #[test]
    fn a_larger_negotiated_mtu_means_fewer_writes() {
        // MTU negotiation is what makes transfers bearable; prove it is used.
        let (link, state) = FakeBleLink::new(244);
        let mut transport = BleTransport::new(Box::new(link));

        transport.write(&vec![0x01; 500]).unwrap();
        assert_eq!(state.lock().unwrap().written_chunks.len(), 3);
    }

    #[test]
    fn reports_its_chunk_limit_so_framing_can_split_too() {
        let (link, _state) = FakeBleLink::new(185);
        let transport = BleTransport::new(Box::new(link));
        assert_eq!(transport.max_write_chunk(), 185);
        assert_eq!(transport.kind(), TransportKind::Ble);
    }

    #[test]
    fn a_zero_mtu_does_not_cause_an_infinite_split() {
        // A stack that reports 0 before negotiation would otherwise produce an
        // empty chunk forever.
        let (link, state) = FakeBleLink::new(0);
        let mut transport = BleTransport::new(Box::new(link));
        state.lock().unwrap().mtu_payload = 0;

        assert!(transport.max_write_chunk() >= 1);
    }

    #[test]
    fn writing_waits_for_device_buffer_space() {
        let (link, state) = FakeBleLink::new(20);
        state.lock().unwrap().free_buffer = Some(1_000);
        let mut transport = BleTransport::new(Box::new(link));

        transport.write(&vec![0x7F; 60]).unwrap();

        let state = state.lock().unwrap();
        assert_eq!(state.written_chunks.len(), 3);
        // The fake decrements its buffer per write, as the device does.
        assert_eq!(state.free_buffer, Some(940));
    }

    #[test]
    fn a_permanently_full_device_buffer_fails_instead_of_dropping_bytes() {
        // Silently overrunning the device is the failure mode this guards: the
        // packets vanish and the RPC stream desynchronises with no error.
        let (link, state) = FakeBleLink::new(20);
        state.lock().unwrap().free_buffer = Some(0);
        let mut transport = BleTransport::new(Box::new(link));

        let err = transport.write(b"anything").unwrap_err();
        assert!(matches!(err, AppError::SerialError(_)), "got {:?}", err);
        assert!(
            state.lock().unwrap().written_chunks.is_empty(),
            "nothing may be written while the device has no room"
        );
    }

    #[test]
    fn a_device_without_flow_control_is_not_throttled() {
        let (link, state) = FakeBleLink::new(20);
        state.lock().unwrap().free_buffer = None;
        let mut transport = BleTransport::new(Box::new(link));

        transport.write(&vec![0x11; 40]).unwrap();
        assert_eq!(state.lock().unwrap().written_chunks.len(), 2);
    }

    #[test]
    fn writing_while_disconnected_is_an_error() {
        let (link, state) = FakeBleLink::new(20);
        state.lock().unwrap().connected = false;
        let mut transport = BleTransport::new(Box::new(link));

        assert!(transport.write(b"hello").is_err());
    }

    #[test]
    fn an_empty_read_means_nothing_yet_not_end_of_stream() {
        let (link, _state) = FakeBleLink::new(20);
        let mut transport = BleTransport::new(Box::new(link));

        let mut buf = [0u8; 32];
        assert_eq!(transport.read(&mut buf).unwrap(), 0);
    }

    #[test]
    fn notifications_are_buffered_across_reads() {
        // A notification larger than the caller's buffer must not lose the tail.
        let (link, state) = FakeBleLink::new(20);
        state.lock().unwrap().to_notify.extend(b"abcdefghij");
        let mut transport = BleTransport::new(Box::new(link));

        let mut buf = [0u8; 4];
        assert_eq!(transport.read(&mut buf).unwrap(), 4);
        assert_eq!(&buf, b"abcd");
        assert_eq!(transport.read(&mut buf).unwrap(), 4);
        assert_eq!(&buf, b"efgh");
        assert_eq!(transport.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf[..2], b"ij");
    }

    #[test]
    fn a_frame_survives_the_full_round_trip_over_ble() {
        // The case the whole module exists for: a frame far larger than the MTU,
        // written in chunks and delivered back a packet at a time.
        let (link, state) = FakeBleLink::new(20);
        let payload = vec![0x5A; 400];
        {
            let mut s = state.lock().unwrap();
            s.notify_granularity = 20;
            s.to_notify.extend(framed(&payload));
        }
        let mut transport = BleTransport::new(Box::new(link));

        write_frame(&mut transport, b"request").unwrap();
        let mut reader = FrameReader::new();
        let frame = read_frame_blocking(&mut transport, &mut reader, 64).unwrap();

        assert_eq!(frame, payload);
        assert!(
            state
                .lock()
                .unwrap()
                .written_chunks
                .iter()
                .all(|c| c.len() <= 20),
            "no write may exceed the MTU"
        );
    }

    #[test]
    fn closing_disconnects_the_link() {
        let (link, state) = FakeBleLink::new(20);
        let mut transport = BleTransport::new(Box::new(link));

        transport.close().unwrap();
        assert!(!state.lock().unwrap().connected);
    }
}
