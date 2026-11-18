//! Shared test support: an in-memory [`Transport`] backed by `vd_netsim::SimLink`.
//!
//! Lives in `tests/` rather than `src/` on purpose: the simulator must not become a
//! production dependency of `vd-net` (it is a lateral crate dependency, which
//! [docs/01_ARCHITECTURE.md] §3 forbids for the product graph). Integration tests
//! depend on `vd-netsim` as a dev-dependency instead.
//!
//! Frames are `[channel_tag: u8][payload…]`. `SendMode` is accepted but not simulated
//! differently yet — `SimLink` has no retransmission model, so reliable and datagram
//! channels both degrade to plain packets. T-0204/T-0206 add real ordering/loss logic.

#![allow(dead_code)] // each integration-test binary uses a subset of these helpers

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use bytes::{BufMut, Bytes, BytesMut};
use vd_common::{Error, Result};
use vd_net::{SendMode, Transport};
use vd_netsim::{LinkConfig, SimLink};
use vd_proto::Channel;

/// Which end of the simulated link an endpoint is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// First endpoint.
    A,
    /// Second endpoint.
    B,
}

#[derive(Debug)]
struct Inner {
    a_to_b: SimLink,
    b_to_a: SimLink,
    inbox_a: VecDeque<Bytes>,
    inbox_b: VecDeque<Bytes>,
}

impl Inner {
    fn lock(inner: &Arc<Mutex<Self>>) -> Result<MutexGuard<'_, Self>> {
        inner
            .lock()
            .map_err(|_| Error::Io("sim transport mutex poisoned".into()))
    }

    fn inbox_mut(&mut self, side: Side) -> &mut VecDeque<Bytes> {
        match side {
            Side::A => &mut self.inbox_a,
            Side::B => &mut self.inbox_b,
        }
    }
}

/// Encode a channel + payload into the simulator's wire frame.
#[must_use]
pub fn encode_packet(channel: Channel, payload: &[u8]) -> Bytes {
    let mut buf = BytesMut::with_capacity(payload.len() + 1);
    buf.put_u8(channel as u8);
    buf.put_slice(payload);
    buf.freeze()
}

/// Decode a simulator wire frame back into a channel + payload.
///
/// # Errors
/// [`Error::Protocol`] for an empty frame or an unknown channel tag.
pub fn decode_packet(frame: &Bytes) -> Result<(Channel, Bytes)> {
    let (tag, rest) = frame
        .split_first()
        .ok_or_else(|| Error::Protocol("empty sim frame".into()))?;
    let channel = channel_from_tag(*tag)
        .ok_or_else(|| Error::Protocol(format!("unknown sim channel tag {tag}")))?;
    Ok((channel, frame.slice_ref(rest)))
}

fn channel_from_tag(tag: u8) -> Option<Channel> {
    // Mirrors `Channel`'s `#[repr(u8)]` discriminants.
    Some(match tag {
        0 => Channel::Control,
        1 => Channel::Input,
        2 => Channel::InputMove,
        3 => Channel::Video,
        4 => Channel::Cursor,
        5 => Channel::Audio,
        6 => Channel::Feedback,
        7 => Channel::Clipboard,
        8 => Channel::File,
        9 => Channel::Chat,
        _ => return None,
    })
}

/// A connected pair of in-memory transports sharing one simulated clock.
#[derive(Debug)]
pub struct SimPair {
    /// Endpoint A.
    pub a: SimTransport,
    /// Endpoint B.
    pub b: SimTransport,
}

impl SimPair {
    /// Build a connected pair. `a_to_b` shapes A→B traffic, `b_to_a` the reverse.
    #[must_use]
    pub fn new(a_to_b: LinkConfig, b_to_a: LinkConfig, seed: u64) -> Self {
        let inner = Arc::new(Mutex::new(Inner {
            a_to_b: SimLink::new(a_to_b, seed | 1),
            b_to_a: SimLink::new(b_to_a, (seed | 1) ^ 0x0005_DEEC_E66D),
            inbox_a: VecDeque::new(),
            inbox_b: VecDeque::new(),
        }));
        Self {
            a: SimTransport {
                side: Side::A,
                inner: Arc::clone(&inner),
                addr: "sim://a",
                closed: false,
            },
            b: SimTransport {
                side: Side::B,
                inner,
                addr: "sim://b",
                closed: false,
            },
        }
    }

    /// Advance simulated time on both directions and move delivered frames to inboxes.
    pub fn advance_ms(&mut self, ms: u64) {
        let Ok(mut inner) = self.a.inner.lock() else {
            panic!("sim transport mutex poisoned");
        };
        inner.a_to_b.advance_ms(ms);
        inner.b_to_a.advance_ms(ms);

        let ready = inner.a_to_b.recv_ready();
        inner.inbox_b.extend(ready);
        let ready = inner.b_to_a.recv_ready();
        inner.inbox_a.extend(ready);
    }
}

/// One end of an in-memory connection implementing the real [`Transport`] trait.
#[derive(Debug)]
pub struct SimTransport {
    side: Side,
    inner: Arc<Mutex<Inner>>,
    addr: &'static str,
    closed: bool,
}

impl SimTransport {
    /// Which side this endpoint is.
    #[must_use]
    pub fn side(&self) -> Side {
        self.side
    }

    fn lock(&self) -> Result<MutexGuard<'_, Inner>> {
        Inner::lock(&self.inner)
    }
}

impl Transport for SimTransport {
    fn send(&mut self, channel: Channel, _mode: SendMode, payload: Bytes) -> Result<()> {
        if self.closed {
            return Err(Error::Io("sim transport closed".into()));
        }
        let frame = encode_packet(channel, &payload);
        let mut inner = self.lock()?;
        let link = match self.side {
            Side::A => &mut inner.a_to_b,
            Side::B => &mut inner.b_to_a,
        };
        link.send(frame)
    }

    fn try_recv(&mut self) -> Result<Option<(Channel, Bytes)>> {
        let raw = {
            let mut inner = self.lock()?;
            inner.inbox_mut(self.side).pop_front()
        };
        raw.map(|frame| decode_packet(&frame)).transpose()
    }

    fn local_addr(&self) -> Result<String> {
        Ok(self.addr.to_owned())
    }

    fn close(&mut self) -> Result<()> {
        self.closed = true;
        Ok(())
    }
}
