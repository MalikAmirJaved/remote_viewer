//! Noise_XX_25519_ChaChaPoly_BLAKE2s handshake and transport (ADR-0004).

use snow::{Builder, HandshakeState, TransportState};
use vd_common::{Error, Result};

use crate::keys::DeviceKeyPair;

/// Canonical Noise pattern for remote_viewer E2E sessions.
pub const NOISE_PATTERN: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";

/// Max Noise message size (protocol limit).
const MAX_MSG_LEN: usize = 65_535;

pub(crate) fn noise_params() -> Result<snow::params::NoiseParams> {
    NOISE_PATTERN
        .parse()
        .map_err(|e| Error::Crypto(format!("parse {NOISE_PATTERN}: {e}")))
}

fn map_snow(err: &snow::Error) -> Error {
    Error::Crypto(err.to_string())
}

fn build_handshake(keys: &DeviceKeyPair, initiator: bool) -> Result<HandshakeState> {
    if keys.secret().len() != 32 {
        return Err(Error::InvalidArgument(format!(
            "static secret must be 32 bytes, got {}",
            keys.secret().len()
        )));
    }
    let builder = Builder::new(noise_params()?).local_private_key(keys.secret());
    if initiator {
        builder.build_initiator().map_err(|e| map_snow(&e))
    } else {
        builder.build_responder().map_err(|e| map_snow(&e))
    }
}

/// In-progress Noise XX handshake (initiator or responder).
///
/// XX message order:
/// 1. initiator → `e`
/// 2. responder → `e, ee, s, es`
/// 3. initiator → `s, se`
pub struct NoiseHandshake {
    state: HandshakeState,
}

impl NoiseHandshake {
    /// Start as Noise XX initiator with the local device static keypair.
    ///
    /// # Errors
    /// Returns [`Error::Crypto`] / [`Error::InvalidArgument`] on setup failure.
    pub fn initiator(keys: &DeviceKeyPair) -> Result<Self> {
        Ok(Self {
            state: build_handshake(keys, true)?,
        })
    }

    /// Start as Noise XX responder with the local device static keypair.
    ///
    /// # Errors
    /// Returns [`Error::Crypto`] / [`Error::InvalidArgument`] on setup failure.
    pub fn responder(keys: &DeviceKeyPair) -> Result<Self> {
        Ok(Self {
            state: build_handshake(keys, false)?,
        })
    }

    /// Whether this side was built as initiator.
    #[must_use]
    pub fn is_initiator(&self) -> bool {
        self.state.is_initiator()
    }

    /// Whether it is this side's turn to [`Self::write_message`].
    #[must_use]
    pub fn is_my_turn(&self) -> bool {
        self.state.is_my_turn()
    }

    /// Whether all handshake messages are done and [`Self::into_transport`] is safe.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.state.is_handshake_finished()
    }

    /// Peer static public key if already revealed by the pattern.
    #[must_use]
    pub fn peer_static_public(&self) -> Option<&[u8]> {
        self.state.get_remote_static()
    }

    /// Write the next handshake message (optional payload is authenticated).
    ///
    /// # Errors
    /// Returns [`Error::Crypto`] on Noise state / MAC failures.
    pub fn write_message(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; MAX_MSG_LEN];
        let len = self
            .state
            .write_message(payload, &mut buf)
            .map_err(|e| map_snow(&e))?;
        buf.truncate(len);
        Ok(buf)
    }

    /// Read the next handshake message; returns the decrypted payload (often empty).
    ///
    /// # Errors
    /// Returns [`Error::Crypto`] if the message is malformed, truncated, or tampered.
    pub fn read_message(&mut self, message: &[u8]) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; MAX_MSG_LEN];
        let len = self
            .state
            .read_message(message, &mut buf)
            .map_err(|e| map_snow(&e))?;
        buf.truncate(len);
        Ok(buf)
    }

    /// Finish the handshake and enter transport mode.
    ///
    /// # Errors
    /// Returns [`Error::Crypto`] if the handshake is incomplete or split fails.
    pub fn into_transport(self) -> Result<NoiseTransport> {
        if !self.state.is_handshake_finished() {
            return Err(Error::Crypto("Noise XX handshake not finished".into()));
        }
        let peer = self
            .state
            .get_remote_static()
            .ok_or_else(|| Error::Crypto("peer static missing after XX".into()))?
            .to_vec();
        let state = self.state.into_transport_mode().map_err(|e| map_snow(&e))?;
        Ok(NoiseTransport {
            state,
            peer_static: peer,
        })
    }
}

/// Post-handshake AEAD transport (ChaChaPoly session keys).
pub struct NoiseTransport {
    state: TransportState,
    peer_static: Vec<u8>,
}

impl NoiseTransport {
    /// Peer's Noise static public key (for TOFU pinning).
    #[must_use]
    pub fn peer_static_public(&self) -> &[u8] {
        &self.peer_static
    }

    /// Encrypt `plaintext` into a Noise transport ciphertext (payload + tag).
    ///
    /// # Errors
    /// Returns [`Error::Crypto`] if the plaintext is too large or encryption fails.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        if plaintext.len() + 16 > MAX_MSG_LEN {
            return Err(Error::InvalidArgument(format!(
                "plaintext too large for Noise message ({} bytes)",
                plaintext.len()
            )));
        }
        let mut buf = vec![0u8; plaintext.len() + 16];
        let len = self
            .state
            .write_message(plaintext, &mut buf)
            .map_err(|e| map_snow(&e))?;
        buf.truncate(len);
        Ok(buf)
    }

    /// Decrypt a Noise transport ciphertext.
    ///
    /// # Errors
    /// Returns [`Error::Crypto`] on authentication failure or bad length.
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() > MAX_MSG_LEN {
            return Err(Error::InvalidArgument(format!(
                "ciphertext exceeds Noise max ({} bytes)",
                ciphertext.len()
            )));
        }
        let mut buf = vec![0u8; ciphertext.len()];
        let len = self
            .state
            .read_message(ciphertext, &mut buf)
            .map_err(|e| map_snow(&e))?;
        buf.truncate(len);
        Ok(buf)
    }
}
