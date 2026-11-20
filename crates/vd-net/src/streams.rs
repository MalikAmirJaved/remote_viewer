//! Reliable QUIC streams for [`Channel::Control`] and [`Channel::Input`] (T-0203).
//!
//! Video, audio, mouse-move, and feedback stay on datagrams (T-0204) so a large
//! frame cannot head-of-line block input. This module only maps the two reliable
//! channels the task asks for.
//!
//! # Stream layout
//!
//! The side that calls [`ReliableChannels::open`] (the viewer, by convention) opens
//! two bidirectional streams and writes a one-byte tag: `Channel as u8`
//! (`Control = 0`, `Input = 1`). [`ReliableChannels::accept`] (the host) reads that
//! tag, so accept order does not matter. Each direction then carries a sequence of
//! length-prefixed opaque payloads:
//!
//! ```text
//! tag: u8                         // once, at the start of the stream
//! repeated: u32be length | bytes  // length ≤ MAX_RELIABLE_PAYLOAD
//! ```
//!
//! Payloads are opaque. Postcard envelopes stay in the app; this layer does not
//! parse them.
//!
//! # Priority
//!
//! Quinn schedules a higher `SendStream` priority first. Input is
//! [`INPUT_STREAM_PRIORITY`], Control is [`CONTROL_STREAM_PRIORITY`], and both sit
//! strictly above `0`, which is left for later bulk reliable streams (clipboard,
//! file, chat). Each side sets the priority on **its own** send half: priority does
//! not cross the connection.
//!
//! [`Channel::Control`]: vd_proto::Channel::Control
//! [`Channel::Input`]: vd_proto::Channel::Input

use quinn::{ClosedStream, ReadExactError, RecvStream, SendStream, VarInt, WriteError};
use vd_common::{Error, Result};
use vd_proto::Channel;

use crate::quic::QuicConnection;

/// Send priority for the Control stream. Higher is scheduled first.
///
/// `0` is reserved for bulk reliable streams opened by later tasks.
pub const CONTROL_STREAM_PRIORITY: i32 = 1;

/// Send priority for the Input stream. Must stay strictly above Control.
pub const INPUT_STREAM_PRIORITY: i32 = 2;

const _: () =
    assert!(CONTROL_STREAM_PRIORITY > 0 && INPUT_STREAM_PRIORITY > CONTROL_STREAM_PRIORITY);

/// Largest payload [`ReliableChannels::send`] will write or [`ReliableChannels::recv`]
/// will allocate.
pub const MAX_RELIABLE_PAYLOAD: usize = 64 * 1024;

/// Application error code reset onto a stream whose tag is not Control or Input.
const BAD_STREAM_TAG: u32 = 1;

/// Control and Input, each on its own bidirectional QUIC stream.
#[derive(Debug)]
pub struct ReliableChannels {
    control: ChannelIo,
    input: ChannelIo,
}

#[derive(Debug)]
struct ChannelIo {
    send: SendStream,
    recv: RecvStream,
}

impl ReliableChannels {
    /// Open Control and Input streams toward the peer.
    ///
    /// The peer must be in [`Self::accept`]. Call this from the viewer after
    /// `QuicClient::connect`.
    ///
    /// # Errors
    /// Connection closed, stream limit exhausted, or the send half closes before the
    /// channel tag is written.
    pub async fn open(conn: &QuicConnection) -> Result<Self> {
        let control = open_one(conn, Channel::Control).await?;
        let input = open_one(conn, Channel::Input).await?;
        Ok(Self { control, input })
    }

    /// Accept the two streams opened by the peer and classify them by their tag.
    ///
    /// Waits until both a Control stream and an Input stream have been tagged.
    /// A second stream for the same channel, or any other tag, is reset and
    /// returned as [`Error::Protocol`].
    ///
    /// # Errors
    /// Connection closed, a short read on the tag, an unknown tag, or a duplicate
    /// channel.
    pub async fn accept(conn: &QuicConnection) -> Result<Self> {
        let mut control = None;
        let mut input = None;
        for _ in 0..2 {
            let (mut send, mut recv) = conn.accept_bi().await?;
            let mut tag = [0u8; 1];
            if let Err(err) = recv.read_exact(&mut tag).await {
                let _ = send.reset(VarInt::from_u32(BAD_STREAM_TAG));
                return Err(read_err(&err));
            }
            let channel = match tag[0] {
                x if x == Channel::Control as u8 => Channel::Control,
                x if x == Channel::Input as u8 => Channel::Input,
                other => {
                    let _ = send.reset(VarInt::from_u32(BAD_STREAM_TAG));
                    return Err(Error::Protocol(format!(
                        "reliable stream: tag {other} is not Control (0) or Input (1)"
                    )));
                }
            };
            if let Err(err) = send.set_priority(priority_of(channel)?) {
                return Err(closed_err(&err));
            }
            let mut io = ChannelIo { send, recv };
            let slot = match channel {
                Channel::Control => &mut control,
                Channel::Input => &mut input,
                _ => unreachable!("tag decode only yields Control or Input"),
            };
            if slot.is_some() {
                let _ = io.send.reset(VarInt::from_u32(BAD_STREAM_TAG));
                return Err(Error::Protocol(format!(
                    "reliable stream: duplicate {channel:?} stream"
                )));
            }
            *slot = Some(io);
        }
        Ok(Self {
            control: control
                .ok_or_else(|| Error::Protocol("reliable stream: Control stream missing".into()))?,
            input: input
                .ok_or_else(|| Error::Protocol("reliable stream: Input stream missing".into()))?,
        })
    }

    /// Priority currently set on the local send half of `channel`.
    ///
    /// # Errors
    /// The channel is not Control or Input, or that send half is already closed.
    pub fn send_priority(&self, channel: Channel) -> Result<i32> {
        let io = self.io(channel)?;
        io.send.priority().map_err(|err| closed_err(&err))
    }

    /// Write one length-prefixed payload on `channel`.
    ///
    /// # Errors
    /// `channel` is not Control or Input, `payload` is longer than
    /// [`MAX_RELIABLE_PAYLOAD`], or the stream write fails.
    pub async fn send(&mut self, channel: Channel, payload: &[u8]) -> Result<()> {
        validate_payload(payload)?;
        let io = self.io_mut(channel)?;
        let len = (payload.len() as u32).to_be_bytes();
        io.send
            .write_all(&len)
            .await
            .map_err(|err| write_err(&err))?;
        if !payload.is_empty() {
            io.send
                .write_all(payload)
                .await
                .map_err(|err| write_err(&err))?;
        }
        Ok(())
    }

    /// Read one length-prefixed payload from `channel`.
    ///
    /// # Errors
    /// `channel` is not Control or Input, the peer closed the stream early, the
    /// declared length exceeds [`MAX_RELIABLE_PAYLOAD`], or the read fails.
    pub async fn recv(&mut self, channel: Channel) -> Result<Vec<u8>> {
        let io = self.io_mut(channel)?;
        let mut len_buf = [0u8; 4];
        io.recv
            .read_exact(&mut len_buf)
            .await
            .map_err(|err| read_err(&err))?;
        let len = u32::from_be_bytes(len_buf) as usize;
        if len > MAX_RELIABLE_PAYLOAD {
            return Err(Error::Protocol(format!(
                "reliable stream: frame length {len} exceeds {MAX_RELIABLE_PAYLOAD}"
            )));
        }
        let mut buf = vec![0u8; len];
        if len > 0 {
            io.recv
                .read_exact(&mut buf)
                .await
                .map_err(|err| read_err(&err))?;
        }
        Ok(buf)
    }

    fn io(&self, channel: Channel) -> Result<&ChannelIo> {
        match channel {
            Channel::Control => Ok(&self.control),
            Channel::Input => Ok(&self.input),
            other => Err(not_reliable(other)),
        }
    }

    fn io_mut(&mut self, channel: Channel) -> Result<&mut ChannelIo> {
        match channel {
            Channel::Control => Ok(&mut self.control),
            Channel::Input => Ok(&mut self.input),
            other => Err(not_reliable(other)),
        }
    }
}

fn priority_of(channel: Channel) -> Result<i32> {
    match channel {
        Channel::Control => Ok(CONTROL_STREAM_PRIORITY),
        Channel::Input => Ok(INPUT_STREAM_PRIORITY),
        other => Err(not_reliable(other)),
    }
}

fn validate_payload(payload: &[u8]) -> Result<()> {
    if payload.len() > MAX_RELIABLE_PAYLOAD {
        return Err(Error::InvalidArgument(format!(
            "reliable stream: payload {} B exceeds {MAX_RELIABLE_PAYLOAD}",
            payload.len()
        )));
    }
    Ok(())
}

fn not_reliable(channel: Channel) -> Error {
    Error::InvalidArgument(format!(
        "reliable stream: {channel:?} is not Control or Input"
    ))
}

fn write_err(err: &WriteError) -> Error {
    Error::Io(format!("quic: reliable stream write: {err}"))
}

fn read_err(err: &ReadExactError) -> Error {
    Error::Io(format!("quic: reliable stream read: {err}"))
}

fn closed_err(err: &ClosedStream) -> Error {
    Error::Io(format!("quic: reliable stream closed: {err}"))
}

async fn open_one(conn: &QuicConnection, channel: Channel) -> Result<ChannelIo> {
    let (mut send, recv) = conn.open_bi().await?;
    send.set_priority(priority_of(channel)?)
        .map_err(|err| closed_err(&err))?;
    send.write_all(&[channel as u8])
        .await
        .map_err(|err| write_err(&err))?;
    Ok(ChannelIo { send, recv })
}
