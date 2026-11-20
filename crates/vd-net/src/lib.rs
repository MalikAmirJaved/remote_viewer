//! Network transport: trait, QUIC (quinn), reliable streams, datagrams, pacer, hole-punch, relay.
//!
//! Hot paths stay sync; tokio is used only around accept/connect orchestration in apps.
#![forbid(unsafe_code)]

use bytes::Bytes;
use vd_common::{Error, Result};
use vd_proto::Channel;

pub mod datagram;
pub mod pacer;
pub mod quic;
pub mod reassemble;
pub mod streams;
pub mod video;

pub use pacer::Pacer;

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

/// Attempt UDP hole punch within `timeout_ms` (stub).
///
/// # Errors
/// Always [`Error::NotImplemented`] in P0.
pub fn hole_punch(_local: &str, _remote_candidates: &[String], _timeout_ms: u64) -> Result<()> {
    Err(Error::NotImplemented("UDP hole punch"))
}
