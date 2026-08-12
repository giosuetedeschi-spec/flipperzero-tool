//! The RPC layer that sits on top of a [`crate::transport::Transport`].
//!
//! Replaces the old approach of writing CLI text and scraping for the `">:"`
//! prompt, which cannot work over BLE and could not carry binary payloads.

pub mod framing;
pub mod session;

pub use session::FlipperSession;
