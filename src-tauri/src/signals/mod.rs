//! The Signal Radar domain: what the Flipper's radios see, normalised.

pub mod actions;
pub mod fap_protocol;
pub mod model;
pub mod store;

pub use actions::{ActionOutcome, DeviceCapabilities, perform};
pub use model::{ActionId, Chip, GeoPoint, Signal, SignalKind, fingerprint};
