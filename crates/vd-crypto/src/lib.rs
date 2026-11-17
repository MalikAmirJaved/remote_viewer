//! End-to-end crypto: Noise-style handshake, key storage, password challenge.
//!
//! See ADR-0004 and [`docs/04_SECURITY.md`](../../docs/04_SECURITY.md).
#![forbid(unsafe_code)]

use vd_common::{Error, Result};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Opaque device static keypair (stub — real Noise keys in later tasks).
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct DeviceKeyPair {
    /// Secret key material (zeroized on drop).
    pub secret: Vec<u8>,
    /// Public key bytes.
    pub public: Vec<u8>,
}

/// Trait for platform key storage (DPAPI / Keychain / libsecret).
pub trait KeyStore: Send + Sync {
    /// Load or create the local device static keypair.
    ///
    /// # Errors
    /// Returns [`Error::NotImplemented`] until a platform backend is wired.
    fn load_or_create_device_keys(&self) -> Result<DeviceKeyPair>;

    /// Persist a TOFU pin for a peer's public key.
    ///
    /// # Errors
    /// Returns storage or validation errors.
    fn pin_peer(&self, peer_id: &str, public_key: &[u8]) -> Result<()>;

    /// Fetch a previously pinned peer key, if any.
    ///
    /// # Errors
    /// Returns storage errors.
    fn get_pinned_peer(&self, peer_id: &str) -> Result<Option<Vec<u8>>>;
}

/// In-memory key store for tests (not for production).
#[derive(Debug, Default)]
pub struct MemoryKeyStore {
    keys: std::sync::Mutex<Option<DeviceKeyPair>>,
    pins: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

impl KeyStore for MemoryKeyStore {
    fn load_or_create_device_keys(&self) -> Result<DeviceKeyPair> {
        let mut guard = self
            .keys
            .lock()
            .map_err(|_| Error::Other("keystore lock poisoned".into()))?;
        if guard.is_none() {
            *guard = Some(DeviceKeyPair {
                secret: vec![0u8; 32],
                public: vec![1u8; 32],
            });
        }
        Ok(guard.as_ref().expect("just inserted").clone())
    }

    fn pin_peer(&self, peer_id: &str, public_key: &[u8]) -> Result<()> {
        let mut pins = self
            .pins
            .lock()
            .map_err(|_| Error::Other("keystore lock poisoned".into()))?;
        pins.insert(peer_id.to_string(), public_key.to_vec());
        Ok(())
    }

    fn get_pinned_peer(&self, peer_id: &str) -> Result<Option<Vec<u8>>> {
        let pins = self
            .pins
            .lock()
            .map_err(|_| Error::Other("keystore lock poisoned".into()))?;
        Ok(pins.get(peer_id).cloned())
    }
}

/// Perform Noise XX handshake (stub).
///
/// # Errors
/// Always returns [`Error::NotImplemented`] in P0.
pub fn noise_handshake_xx(_initiator: bool, _keys: &DeviceKeyPair) -> Result<Vec<u8>> {
    Err(Error::NotImplemented(
        "Noise_XX_25519_ChaChaPoly_BLAKE2s handshake",
    ))
}

/// Verify password via challenge-response inside the encrypted channel (stub).
///
/// # Errors
/// Always returns [`Error::NotImplemented`] in P0.
pub fn password_challenge_verify(_challenge: &[u8], _response: &[u8]) -> Result<bool> {
    Err(Error::NotImplemented("Argon2id password challenge"))
}
