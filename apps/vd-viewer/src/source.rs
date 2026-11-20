//! Unified video source for the viewer window (loopback or LAN).

use crate::lan::{LanClientConfig, LanSource};
use crate::loopback::{LoopbackConfig, LoopbackSource};
use vd_codec::DecodedFrame;
use vd_common::Result;

/// Either the in-process loopback pipeline or a direct LAN receive path.
#[derive(Debug)]
pub enum VideoSource {
    /// Fake capture → encode → decode in-process.
    Loopback(LoopbackSource),
    /// QUIC receive → reassemble → decode.
    Lan(LanSource),
}

impl VideoSource {
    pub fn start_loopback(cfg: LoopbackConfig) -> Result<Self> {
        Ok(Self::Loopback(LoopbackSource::start(cfg)?))
    }

    pub fn start_lan(cfg: LanClientConfig) -> Result<Self> {
        Ok(Self::Lan(LanSource::start(cfg)?))
    }

    #[must_use]
    pub fn try_recv_latest(&self) -> Option<DecodedFrame> {
        match self {
            Self::Loopback(s) => s.try_recv_latest(),
            Self::Lan(s) => s.try_recv_latest(),
        }
    }

    #[must_use]
    pub fn produced(&self) -> u64 {
        match self {
            Self::Loopback(s) => s.produced(),
            Self::Lan(s) => s.produced(),
        }
    }

    #[must_use]
    pub fn dropped(&self) -> u64 {
        match self {
            Self::Loopback(s) => s.dropped(),
            Self::Lan(s) => s.dropped(),
        }
    }

    #[must_use]
    pub fn last_error(&self) -> Option<String> {
        match self {
            Self::Loopback(s) => s.last_error(),
            Self::Lan(s) => s.last_error(),
        }
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        match self {
            Self::Loopback(s) => s.is_finished(),
            Self::Lan(s) => s.is_finished(),
        }
    }

    pub fn stop(&mut self) {
        match self {
            Self::Loopback(s) => s.stop(),
            Self::Lan(s) => s.stop(),
        }
    }

    #[must_use]
    pub fn mode_label(&self) -> &'static str {
        match self {
            Self::Loopback(_) => "local loopback",
            Self::Lan(_) => "LAN connect",
        }
    }
}
