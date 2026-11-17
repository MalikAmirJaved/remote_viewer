//! Shared configuration defaults (agent/viewer/server override locally).

use serde::{Deserialize, Serialize};

/// Cross-cutting defaults shared by agent and viewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonConfig {
    /// Protocol version this build speaks.
    pub protocol_version: u16,
    /// Default MTU baseline for datagram framing (bytes).
    pub mtu_bytes: u16,
    /// Max video frames in flight on the sender (latest-frame-wins).
    pub max_frames_in_flight: u8,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            protocol_version: vd_proto_version(),
            mtu_bytes: 1200,
            max_frames_in_flight: 2,
        }
    }
}

/// Keep in sync with `vd_proto::PROTOCOL_VERSION` — duplicated here to avoid a
/// circular dependency (vd-proto depends on vd-common).
fn vd_proto_version() -> u16 {
    1
}
