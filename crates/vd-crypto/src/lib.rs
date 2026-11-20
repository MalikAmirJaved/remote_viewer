//! End-to-end crypto: Noise XX handshake, key storage, password challenge.
//!
//! Pattern: [`NOISE_PATTERN`] (`Noise_XX_25519_ChaChaPoly_BLAKE2s`) via [`snow`]
//! per [ADR-0004](../../docs/adr/0004-e2e-crypto-noise.md) and
//! [`docs/04_SECURITY.md`](../../docs/04_SECURITY.md).
#![forbid(unsafe_code)]

mod keys;
mod noise;

pub use keys::{DeviceKeyPair, KeyStore, MemoryKeyStore};
pub use noise::{NoiseHandshake, NoiseTransport, NOISE_PATTERN};

use vd_common::{Error, Result};

/// Verify password via challenge-response inside the encrypted channel (stub).
///
/// # Errors
/// Always returns [`Error::NotImplemented`] until T-0306.
pub fn password_challenge_verify(_challenge: &[u8], _response: &[u8]) -> Result<bool> {
    Err(Error::NotImplemented("Argon2id password challenge"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_handshake(
        initiator_keys: &DeviceKeyPair,
        responder_keys: &DeviceKeyPair,
    ) -> Result<(NoiseTransport, NoiseTransport)> {
        let mut init = NoiseHandshake::initiator(initiator_keys)?;
        let mut resp = NoiseHandshake::responder(responder_keys)?;

        // -> e
        let msg1 = init.write_message(&[])?;
        let _ = resp.read_message(&msg1)?;

        // <- e, ee, s, es
        let msg2 = resp.write_message(&[])?;
        let _ = init.read_message(&msg2)?;

        // -> s, se
        let msg3 = init.write_message(&[])?;
        let _ = resp.read_message(&msg3)?;

        assert!(init.is_finished());
        assert!(resp.is_finished());

        Ok((init.into_transport()?, resp.into_transport()?))
    }

    #[test]
    fn xx_handshake_shared_keys_encrypt_roundtrip() {
        let store_a = MemoryKeyStore::default();
        let store_b = MemoryKeyStore::default();
        let keys_a = store_a.load_or_create_device_keys().expect("a keys");
        let keys_b = store_b.load_or_create_device_keys().expect("b keys");

        let (mut init_t, mut resp_t) = complete_handshake(&keys_a, &keys_b).expect("handshake");

        assert_eq!(init_t.peer_static_public(), keys_b.public());
        assert_eq!(resp_t.peer_static_public(), keys_a.public());

        let ct = init_t.encrypt(b"hello session").expect("encrypt");
        let pt = resp_t.decrypt(&ct).expect("decrypt");
        assert_eq!(pt, b"hello session");

        let ct2 = resp_t.encrypt(b"ack").expect("encrypt reply");
        let pt2 = init_t.decrypt(&ct2).expect("decrypt reply");
        assert_eq!(pt2, b"ack");
    }

    #[test]
    fn xx_handshake_rejects_tampered_message() {
        let keys_a = DeviceKeyPair::generate().expect("a");
        let keys_b = DeviceKeyPair::generate().expect("b");

        let mut init = NoiseHandshake::initiator(&keys_a).expect("init");
        let mut resp = NoiseHandshake::responder(&keys_b).expect("resp");

        // -> e (no MAC; tampering just changes the ephemeral)
        let msg1 = init.write_message(&[]).expect("msg1");
        let _ = resp.read_message(&msg1).expect("read msg1");

        // <- e, ee, s, es (MAC'd) — flip a ciphertext byte
        let mut msg2 = resp.write_message(&[]).expect("msg2");
        let last = msg2.len() - 1;
        msg2[last] ^= 0x5a;
        let err = init
            .read_message(&msg2)
            .expect_err("tampered handshake must fail");
        assert!(
            matches!(err, Error::Crypto(_)),
            "expected Crypto, got {err:?}"
        );
    }

    #[test]
    fn xx_handshake_rejects_wrong_peer_static_on_transport() {
        let keys_a = DeviceKeyPair::generate().expect("a");
        let keys_b = DeviceKeyPair::generate().expect("b");
        let keys_c = DeviceKeyPair::generate().expect("c");

        let (mut init_t, mut resp_t) = complete_handshake(&keys_a, &keys_b).expect("handshake");
        let (_other_init, mut other_resp) =
            complete_handshake(&keys_a, &keys_c).expect("other session");

        let ct = init_t.encrypt(b"secret").expect("encrypt");
        let err = other_resp
            .decrypt(&ct)
            .expect_err("foreign session must not decrypt");
        assert!(matches!(err, Error::Crypto(_)));

        // Sanity: correct peer still works.
        let pt = resp_t.decrypt(&ct).expect("decrypt");
        assert_eq!(pt, b"secret");
    }

    #[test]
    fn memory_keystore_pins_and_reuses_keys() {
        let store = MemoryKeyStore::default();
        let k1 = store.load_or_create_device_keys().expect("create");
        let k2 = store.load_or_create_device_keys().expect("reload");
        assert_eq!(k1.public(), k2.public());
        assert_eq!(k1.secret(), k2.secret());
        assert_ne!(k1.public(), &[0u8; 32]);

        store.pin_peer("peer-1", k1.public()).expect("pin");
        let pinned = store.get_pinned_peer("peer-1").expect("get");
        assert_eq!(pinned.as_deref(), Some(k1.public()));
        assert!(store
            .get_pinned_peer("missing")
            .expect("get missing")
            .is_none());
    }

    #[test]
    fn device_keypair_rejects_bad_secret_len() {
        let err = DeviceKeyPair::from_secret(&[1u8; 16]).expect_err("len");
        assert!(matches!(err, Error::InvalidArgument(_)));
    }
}
