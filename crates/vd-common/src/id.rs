//! Identifier types for devices, peers, and sessions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// 9-digit style peer ID shown to users (stored as u64 for wire efficiency).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub u64);

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display as zero-padded 9 digits when in range; otherwise raw.
        if self.0 < 1_000_000_000 {
            write!(f, "{:09}", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

/// Stable device identity key fingerprint / UUID-like id (opaque string for now).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(pub String);

/// Session identifier assigned after consent + handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub u128);
