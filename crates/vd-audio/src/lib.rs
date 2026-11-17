//! Audio capture, playback, and Opus encode/decode stubs (`cpal` + Opus later).
#![forbid(unsafe_code)]

use vd_common::{Error, Result};

/// Captures system/loopback or mic PCM (stub).
#[derive(Debug, Default)]
pub struct AudioCapture;

impl AudioCapture {
    /// Start capture.
    ///
    /// # Errors
    /// [`Error::NotImplemented`] in P0.
    #[allow(clippy::unused_self)]
    pub fn start(&mut self) -> Result<()> {
        Err(Error::NotImplemented("AudioCapture::start (cpal)"))
    }

    /// Read an Opus-ready PCM chunk (stub).
    ///
    /// # Errors
    /// [`Error::NotImplemented`] in P0.
    #[allow(clippy::unused_self)]
    pub fn read_pcm(&mut self) -> Result<Vec<i16>> {
        Err(Error::NotImplemented("AudioCapture::read_pcm"))
    }
}

/// Plays PCM on the viewer (stub).
#[derive(Debug, Default)]
pub struct AudioPlayback;

impl AudioPlayback {
    /// Start playback device.
    ///
    /// # Errors
    /// [`Error::NotImplemented`] in P0.
    #[allow(clippy::unused_self)]
    pub fn start(&mut self) -> Result<()> {
        Err(Error::NotImplemented("AudioPlayback::start (cpal)"))
    }

    /// Queue PCM for playback.
    ///
    /// # Errors
    /// [`Error::NotImplemented`] in P0.
    #[allow(clippy::unused_self)]
    pub fn write_pcm(&mut self, _pcm: &[i16]) -> Result<()> {
        Err(Error::NotImplemented("AudioPlayback::write_pcm"))
    }
}

/// Opus encode stub.
///
/// # Errors
/// Always [`Error::NotImplemented`].
pub fn opus_encode(_pcm: &[i16]) -> Result<Vec<u8>> {
    Err(Error::NotImplemented("opus encode"))
}

/// Opus decode stub.
///
/// # Errors
/// Always [`Error::NotImplemented`].
pub fn opus_decode(_packet: &[u8]) -> Result<Vec<i16>> {
    Err(Error::NotImplemented("opus decode"))
}
