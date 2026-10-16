//! Beast binary framing for Mode-S / ADS-B feeds.
//!
//! Protocol (Mode-S Beast):
//! - `0x1a` escape prefix; `0x1a 0x1a` = literal `0x1a` in payload
//! - `0x31` Mode-AC (2 byte payload)
//! - `0x32` Mode-S short (7 bytes)
//! - `0x33` Mode-S long (14 bytes)
//! - `0x34` status (ignored for decoding)
//!
//! Beast-Reduce / Plus: same framing; Plus adds UUID handshake on connect.

mod client;
mod forward;
mod frame;
mod reduce;
mod server;

pub use client::{BeastClient, BeastSource};
pub use forward::{spawn_forwarder, ForwardOpts};
pub use frame::{BeastFrame, BeastMessageType};
pub use reduce::{normalize_uuid, FeedMode};
pub use server::BeastOutServer;
