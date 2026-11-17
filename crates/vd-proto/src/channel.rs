//! Logical channels and their delivery semantics.

use serde::{Deserialize, Serialize};

/// Logical multiplex channel on the transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Channel {
    /// Reliable, ordered: Hello/Auth/Capabilities, session, quality, monitors, ping.
    Control = 0,
    /// Reliable, ordered, high priority: keys, buttons, wheel, text, Ctrl+Alt+Del request.
    Input = 1,
    /// Unreliable datagram, latest-wins: mouse position.
    InputMove = 2,
    /// Unreliable datagram: video frame fragments.
    Video = 3,
    /// Reliable: cursor shape + hotspot.
    Cursor = 4,
    /// Unreliable datagram: Opus audio frames.
    Audio = 5,
    /// Unreliable datagram ~20 Hz: receiver reports and NACKs.
    Feedback = 6,
    /// Reliable stream: clipboard payloads.
    Clipboard = 7,
    /// Reliable stream per transfer: file chunks.
    File = 8,
    /// Reliable: chat text.
    Chat = 9,
}

impl Channel {
    /// Whether this channel expects reliable ordered delivery.
    #[must_use]
    pub const fn is_reliable(self) -> bool {
        matches!(
            self,
            Self::Control | Self::Input | Self::Cursor | Self::Clipboard | Self::File | Self::Chat
        )
    }

    /// Whether this channel is high-priority (must not wait behind video).
    #[must_use]
    pub const fn is_high_priority(self) -> bool {
        matches!(self, Self::Input | Self::Control)
    }
}
