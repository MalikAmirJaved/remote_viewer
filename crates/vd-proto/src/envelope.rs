//! Versioned message envelope.

use crate::channel::Channel;
use crate::messages::Payload;
use crate::version::PROTOCOL_VERSION;
use serde::{Deserialize, Serialize};
use vd_common::Error;

/// Wire envelope wrapping every multiplexed message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// Protocol version of this message.
    pub version: u16,
    /// Logical channel.
    pub channel: Channel,
    /// Per-channel sequence number.
    pub seq: u64,
    /// Sender monotonic timestamp (µs), for latency telemetry.
    pub ts_us: u64,
    /// Channel-specific payload.
    pub payload: Payload,
}

impl Envelope {
    /// Build an envelope with the current [`PROTOCOL_VERSION`].
    #[must_use]
    pub fn new(channel: Channel, seq: u64, ts_us: u64, payload: Payload) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            channel,
            seq,
            ts_us,
            payload,
        }
    }

    /// Serialize with postcard.
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        postcard::to_allocvec(self).map_err(|e| Error::Protocol(e.to_string()))
    }

    /// Deserialize with postcard.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        postcard::from_bytes(bytes).map_err(|e| Error::Protocol(e.to_string()))
    }
}
