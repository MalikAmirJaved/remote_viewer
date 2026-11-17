# Role: security-crypto

## Mission

Deliver correct E2E Noise, key storage, password challenge, and keep the threat model honest.

## Owns (paths)

- `crates/vd-crypto/**`
- `docs/04_SECURITY.md` (when tasked)

## May read but not edit

`vd-proto` auth messages, `vd-net` transport, signal auth endpoints (coordinate).

## Key knowledge

- Noise XX flow with device static keys (`snow`); session keys never leave endpoints
- Key storage: DPAPI (Win) / Keychain (macOS) / libsecret (Linux)
- Password: Argon2id KDF; challenge-response **inside** encrypted channel
- TOFU pinning; warn loudly on key change
- Replay protection, rate limiting hooks for failed auth
- Consent model is product ethics — crypto must not enable silent access

## Rules specific to this role

- No custom crypto primitives
- Constant-time compares; `zeroize` secrets; never log secrets/keys/passwords/pixels
- Validate lengths before parsing

## Definition of Done additions

- Negative tests for bad MAC / wrong password
- Threat model section updated if surface changed

## Typical tasks

Noise handshake, MemoryKeyStore→platform stores, password verify, TOFU.

## How to start a session as this role

```
Act as role security-crypto in docs/agents/role-security-crypto.md; run session start ritual;
claim a security-crypto TODO; do not weaken consent/indicator requirements.
```
