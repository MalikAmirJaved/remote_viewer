//! Network transport: trait, QUIC (quinn) impl (later), hole-punch, relay client, pacer.
//!
//! Hot paths stay sync; tokio is used only around accept/connect orchestration in apps.
#![forbid(unsafe_code)]

use bytes::Bytes;
use vd_common::{Error, Result};
use vd_proto::Channel;

pub mod quic;

/// Destination / path selection hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendMode {
    /// Reliable ordered stream for this channel.
    Reliable,
    /// Unreliable datagram (latest-wins semantics applied by caller).
    Datagram,
}

/// Abstract transport so QUIC can be swapped for WebRTC (`str0m`) via ADR.
pub trait Transport: Send {
    /// Send a framed payload on a logical channel.
    ///
    /// # Errors
    /// Returns I/O or not-implemented errors.
    fn send(&mut self, channel: Channel, mode: SendMode, payload: Bytes) -> Result<()>;

    /// Non-blocking receive of the next payload, if any.
    ///
    /// # Errors
    /// Returns I/O errors.
    fn try_recv(&mut self) -> Result<Option<(Channel, Bytes)>>;

    /// Local bound address string for diagnostics.
    fn local_addr(&self) -> Result<String>;

    /// Close gracefully.
    ///
    /// # Errors
    /// Returns I/O errors on close failure.
    fn close(&mut self) -> Result<()>;
}

/// Stub transport that always returns [`Error::NotImplemented`].
#[derive(Debug, Default)]
pub struct StubTransport;

impl Transport for StubTransport {
    fn send(&mut self, _channel: Channel, _mode: SendMode, _payload: Bytes) -> Result<()> {
        Err(Error::NotImplemented("QUIC Transport::send"))
    }

    fn try_recv(&mut self) -> Result<Option<(Channel, Bytes)>> {
        Err(Error::NotImplemented("QUIC Transport::try_recv"))
    }

    fn local_addr(&self) -> Result<String> {
        Err(Error::NotImplemented("Transport::local_addr"))
    }

    fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Token-bucket pacer stub (real impl in later tasks).
#[derive(Debug, Clone)]
pub struct Pacer {
    /// Target bitrate bps.
    pub bitrate_bps: u32,
}

impl Pacer {
    /// Create a pacer with a target bitrate.
    #[must_use]
    pub fn new(bitrate_bps: u32) -> Self {
        Self { bitrate_bps }
    }

    /// Returns whether a packet of `size` bytes may be sent now (stub: always true).
    #[must_use]
    pub fn allow(&mut self, _size: usize) -> bool {
        true
    }
}

/// Attempt UDP hole punch within `timeout_ms` (stub).
///
/// # Errors
/// Always [`Error::NotImplemented`] in P0.
pub fn hole_punch(_local: &str, _remote_candidates: &[String], _timeout_ms: u64) -> Result<()> {
    Err(Error::NotImplemented("UDP hole punch"))
}
