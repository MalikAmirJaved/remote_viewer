//! Device static keypairs and key storage.

use snow::params::DHChoice;
use snow::resolvers::{CryptoResolver, DefaultResolver};
use snow::Builder;
use vd_common::{Error, Result};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::noise::noise_params;

/// X25519 private-key length used by Noise 25519.
pub const STATIC_KEY_LEN: usize = 32;

/// Opaque device static keypair for Noise XX.
///
/// Secret material is zeroized on drop. Do not log [`Self::secret`].
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct DeviceKeyPair {
    secret: Vec<u8>,
    public: Vec<u8>,
}

impl std::fmt::Debug for DeviceKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceKeyPair")
            .field("public", &self.public)
            .field("secret", &"<redacted>")
            .finish()
    }
}

impl DeviceKeyPair {
    /// Generate a fresh X25519 static keypair via snow's resolver.
    ///
    /// # Errors
    /// Returns [`Error::Crypto`] if key generation fails.
    pub fn generate() -> Result<Self> {
        let kp = Builder::new(noise_params()?)
            .generate_keypair()
            .map_err(|e| Error::Crypto(format!("generate_keypair: {e}")))?;
        Ok(Self {
            secret: kp.private,
            public: kp.public,
        })
    }

    /// Build a keypair from a 32-byte X25519 private key.
    ///
    /// # Errors
    /// Returns [`Error::InvalidArgument`] if `secret` is not 32 bytes, or
    /// [`Error::Crypto`] if the public key cannot be derived.
    pub fn from_secret(secret: &[u8]) -> Result<Self> {
        if secret.len() != STATIC_KEY_LEN {
            return Err(Error::InvalidArgument(format!(
                "static secret must be {STATIC_KEY_LEN} bytes, got {}",
                secret.len()
            )));
        }
        let public = derive_x25519_public(secret)?;
        Ok(Self {
            secret: secret.to_vec(),
            public,
        })
    }

    /// Static private key bytes (32).
    #[must_use]
    pub fn secret(&self) -> &[u8] {
        &self.secret
    }

    /// Static public key bytes (32).
    #[must_use]
    pub fn public(&self) -> &[u8] {
        &self.public
    }
}

fn derive_x25519_public(secret: &[u8]) -> Result<Vec<u8>> {
    let resolver = DefaultResolver;
    let mut dh = resolver
        .resolve_dh(&DHChoice::Curve25519)
        .ok_or_else(|| Error::Crypto("snow DH resolver unavailable".into()))?;
    if secret.len() != dh.priv_len() {
        return Err(Error::InvalidArgument(format!(
            "secret len {} != dh priv_len {}",
            secret.len(),
            dh.priv_len()
        )));
    }
    dh.set(secret);
    Ok(dh.pubkey().to_vec())
}

/// Trait for platform key storage (DPAPI / Keychain / libsecret).
pub trait KeyStore: Send + Sync {
    /// Load or create the local device static keypair.
    ///
    /// # Errors
    /// Returns storage or crypto errors.
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
            *guard = Some(DeviceKeyPair::generate()?);
        }
        Ok(guard.as_ref().expect("just inserted").clone())
    }

    fn pin_peer(&self, peer_id: &str, public_key: &[u8]) -> Result<()> {
        if public_key.len() != STATIC_KEY_LEN {
            return Err(Error::InvalidArgument(format!(
                "peer public key must be {STATIC_KEY_LEN} bytes, got {}",
                public_key.len()
            )));
        }
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
