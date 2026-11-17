//! Wire protocol for remote_viewer: versioned envelopes, channels, messages.
//!
//! See [`docs/02_PROTOCOL.md`](../../docs/02_PROTOCOL.md).
#![forbid(unsafe_code)]

pub mod channel;
pub mod envelope;
pub mod messages;
pub mod version;

pub use channel::Channel;
pub use envelope::Envelope;
pub use messages::*;
pub use version::PROTOCOL_VERSION;
