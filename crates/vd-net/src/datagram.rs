//! Unreliable QUIC datagrams for Video / Feedback / InputMove / Audio (T-0204).
//!
//! Reliable Control and Input stay on streams ([`crate::streams`]). Datagrams are
//! unordered and may be lost; the sender uses quinn's drop-oldest send buffer so a
//! congested path prefers newer video over stale fragments.
//!
//! # Wire layout
//!
//! ```text
//! tag: u8          // Channel discriminant (InputMove=2, Video=3, Audio=5, Feedback=6)
//! payload: bytes   // opaque; length ≤ max_payload()
//! ```
//!
//! Postcard envelopes stay in the app. This layer only multiplexes the channel tag.
//!
//! # Size limits
//!
//! [`DEFAULT_MAX_DATAGRAM_PAYLOAD`] (1150 B) matches the video fragment data budget so a
//! fragment always fits one datagram on the protocol's 1200 B path MTU. [`DatagramIo::send`]
//! also refuses anything larger than the live QUIC `max_datagram_size() - 1`.

use bytes::{Bytes, BytesMut};
use quinn::SendDatagramError;
use vd_common::{Error, Result};
use vd_proto::Channel;

use crate::quic::QuicConnection;
use crate::video::DEFAULT_MAX_FRAGMENT_DATA;

/// Soft cap on datagram application payload (bytes after the channel tag).
///
/// Equal to [`DEFAULT_MAX_FRAGMENT_DATA`] so Video fragments need no further split at
/// this layer.
pub const DEFAULT_MAX_DATAGRAM_PAYLOAD: usize = DEFAULT_MAX_FRAGMENT_DATA;

/// Default quinn outgoing datagram buffer (bytes of payload + overhead).
pub const DEFAULT_DATAGRAM_SEND_BUFFER: usize = 1024 * 1024;

/// Default quinn incoming datagram buffer (bytes of payload + overhead).
pub const DEFAULT_DATAGRAM_RECV_BUFFER: usize = 1024 * 1024;

/// Send and receive tagged unreliable datagrams on one QUIC connection.
#[derive(Debug, Clone)]
pub struct DatagramIo {
    conn: QuicConnection,
}

impl DatagramIo {
    /// Wrap an established connection.
    #[must_use]
    pub fn new(conn: QuicConnection) -> Self {
        Self { conn }
    }

    /// Underlying connection (for closing or sharing with reliable streams).
    #[must_use]
    pub fn connection(&self) -> &QuicConnection {
        &self.conn
    }

    /// Largest payload [`Self::send`] will accept right now.
    ///
    /// The lesser of [`DEFAULT_MAX_DATAGRAM_PAYLOAD`] and `max_datagram_size() - 1`.
    /// `None` if the peer has not enabled datagrams.
    #[must_use]
    pub fn max_payload(&self) -> Option<usize> {
        let quic = self.conn.inner().max_datagram_size()?.saturating_sub(1);
        Some(quic.min(DEFAULT_MAX_DATAGRAM_PAYLOAD))
    }

    /// Transmit one tagged datagram. Prefer this over waiting: under congestion quinn
    /// drops older unsent datagrams so newer video wins.
    ///
    /// # Errors
    /// `channel` is not a datagram channel, `payload` exceeds [`Self::max_payload`],
    /// datagrams are disabled, or the connection is lost.
    pub fn send(&self, channel: Channel, payload: &[u8]) -> Result<()> {
        ensure_datagram_channel(channel)?;
        let max = self.max_payload().ok_or_else(|| {
            Error::Unavailable("quic: peer or local config disabled datagrams".into())
        })?;
        if payload.len() > max {
            return Err(Error::InvalidArgument(format!(
                "datagram: payload {} B exceeds max {max} B for {channel:?}",
                payload.len()
            )));
        }
        let mut buf = BytesMut::with_capacity(1 + payload.len());
        buf.extend_from_slice(&[channel as u8]);
        buf.extend_from_slice(payload);
        self.conn
            .inner()
            .send_datagram(buf.freeze())
            .map_err(send_err)
    }

    /// Wait for the next tagged datagram.
    ///
    /// # Errors
    /// Connection closed, empty datagram, or an unknown / non-datagram channel tag.
    pub async fn recv(&self) -> Result<(Channel, Bytes)> {
        let raw = self
            .conn
            .inner()
            .read_datagram()
            .await
            .map_err(|e| Error::Io(format!("quic: datagram recv: {e}")))?;
        if raw.is_empty() {
            return Err(Error::Protocol(
                "datagram: empty (missing channel tag)".into(),
            ));
        }
        let channel = decode_datagram_channel(raw[0])?;
        Ok((channel, raw.slice(1..)))
    }
}

fn ensure_datagram_channel(channel: Channel) -> Result<()> {
    if is_datagram_channel(channel) {
        Ok(())
    } else {
        Err(Error::InvalidArgument(format!(
            "datagram: {channel:?} is not a datagram channel"
        )))
    }
}

fn is_datagram_channel(channel: Channel) -> bool {
    matches!(
        channel,
        Channel::InputMove | Channel::Video | Channel::Audio | Channel::Feedback
    )
}

fn decode_datagram_channel(tag: u8) -> Result<Channel> {
    let channel = match tag {
        x if x == Channel::InputMove as u8 => Channel::InputMove,
        x if x == Channel::Video as u8 => Channel::Video,
        x if x == Channel::Audio as u8 => Channel::Audio,
        x if x == Channel::Feedback as u8 => Channel::Feedback,
        other => {
            return Err(Error::Protocol(format!(
                "datagram: tag {other} is not a datagram channel"
            )));
        }
    };
    Ok(channel)
}

fn send_err(err: SendDatagramError) -> Error {
    match err {
        SendDatagramError::TooLarge => {
            Error::InvalidArgument("datagram: payload larger than peer max_datagram_size".into())
        }
        SendDatagramError::UnsupportedByPeer => {
            Error::Unavailable("quic: peer does not support datagrams".into())
        }
        SendDatagramError::Disabled => Error::Unavailable("quic: local datagrams disabled".into()),
        SendDatagramError::ConnectionLost(e) => {
            Error::Io(format!("quic: datagram send connection lost: {e}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_datagram_channel, is_datagram_channel, DEFAULT_MAX_DATAGRAM_PAYLOAD};
    use crate::video::DEFAULT_MAX_FRAGMENT_DATA;
    use vd_proto::Channel;

    #[test]
    fn soft_cap_matches_fragment_budget() {
        assert_eq!(DEFAULT_MAX_DATAGRAM_PAYLOAD, DEFAULT_MAX_FRAGMENT_DATA);
    }

    #[test]
    fn only_unreliable_channels_are_datagrams() {
        assert!(is_datagram_channel(Channel::Video));
        assert!(is_datagram_channel(Channel::Feedback));
        assert!(is_datagram_channel(Channel::InputMove));
        assert!(is_datagram_channel(Channel::Audio));
        assert!(!is_datagram_channel(Channel::Control));
        assert!(!is_datagram_channel(Channel::Input));
        assert!(!is_datagram_channel(Channel::Clipboard));
    }

    #[test]
    fn datagram_tags_decode() {
        assert_eq!(
            decode_datagram_channel(Channel::Video as u8).expect("video"),
            Channel::Video
        );
        assert!(decode_datagram_channel(Channel::Control as u8).is_err());
        assert!(decode_datagram_channel(0xFF).is_err());
    }
}
