# Security — remote_viewer

## Threat model (summary)

| Asset | Threat | Mitigation |
|-------|--------|------------|
| Screen contents / input | Eavesdrop on path or servers | E2E Noise; relay sees ciphertext only |
| Password | Interception / reuse | Challenge-response inside encrypted channel; Argon2id; never logged |
| Device keys | Theft from disk | Platform keystore (DPAPI/Keychain/libsecret); TOFU pins |
| Host machine | Silent remote control | Visible indicator; consent; host can disconnect; no stealth |
| Signal server | Malicious broker | Cannot decrypt media; rate-limit; auth for ID claiming |
| Viewer | Malicious host | User chooses whom to connect; clear UI; no auto-run of remote files |

**Out of scope for v1:** nation-state host compromise, DMA attacks, evil maid with physical access.

## E2E encryption

- Noise `Noise_XX_25519_ChaChaPoly_BLAKE2s` ([ADR-0004](adr/0004-e2e-crypto-noise.md)).
- Crate API (`vd-crypto`): `NoiseHandshake` / `NoiseTransport` over `snow`; device
  static keys via `DeviceKeyPair` + `KeyStore` (in-memory for tests; platform stores later).
- Signal/relay never hold session keys.
- Replay protection via handshake ephemeral + session nonces (Noise transport nonces).

## Authentication

1. Device static keys (Noise XX).
2. TOFU pin on first successful auth; warn on key change.
3. Password challenge **after** Noise completes.
4. Unattended mode: separate permanent password / approved device list, **explicit opt-in**.

## Consent and visibility (ethics)

- Interactive sessions: host **must** see consent prompt (unless unattended opt-in).
- While connected: **always-on visible indicator**; one-click end session.
- No hidden mode, no process disguising, no stealth persistence, no credential harvesting.
- Clipboard and file transfer: size limits + consent for large/sensitive payloads.

## Abuse prevention

- Signal: rate-limit ID lookups, connection attempts, candidate spam.
- Relay: per-session bandwidth caps; idle timeouts; no packet inspection beyond routing headers.
- Agent: refuse connections that fail auth; backoff on repeated failures.

## Implementation rules

- No custom crypto; use `snow`, `argon2`, `blake3`, `zeroize`.
- Constant-time compare for auth proofs.
- Never log secrets, keys, passwords, or raw pixels.
- Validate all lengths from the wire before allocate/copy.
- `cargo deny` + fuzz proto parsers (P8).

## Related

[02_PROTOCOL.md](02_PROTOCOL.md) · role [agents/role-security-crypto.md](agents/role-security-crypto.md) · Cursor rule `30-security-crypto`
