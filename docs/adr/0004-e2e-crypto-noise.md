# 0004 — E2E crypto: Noise XX

- **Status:** Accepted
- **Date:** 2026-11-17
- **Deciders:** cursor-init (architect)

## Context

Signal and relay servers must never see session plaintext. Password must not cross the
network in cleartext. Peers need TOFU key pinning.

## Decision

- End-to-end: **Noise** pattern `Noise_XX_25519_ChaChaPoly_BLAKE2s` via **`snow`**.
- Device static keypairs; TOFU pin known peers in platform key stores.
- Password verified by **challenge-response inside** the encrypted channel; secret derived
  with **Argon2id**; never sent in clear.
- Relay/signal forward/broker only opaque bytes / session metadata — no decrypt capability.
- No custom crypto primitives; constant-time compares; `zeroize` secrets.

## Consequences

### Positive

- Clear threat boundary; servers are untrusted for confidentiality.

### Negative / risks

- Noise + QUIC TLS stacking needs careful design (app-layer Noise over QUIC datagrams/streams
  is the v1 intent; avoid double-complexity — document exact layering in P3 tasks).

### Follow-ups

- OPEN QUESTION logged in HANDOFF: Noise over QUIC application data vs relying on QUIC TLS
  with pinned certs. Default: **Noise for E2E** regardless of transport TLS.

## Alternatives considered

1. **TLS only with pinned certs** — servers terminating TLS would see plaintext if mis-deployed; rejected as sole protection.
2. **Signal Double Ratchet** — overkill for desktop remote-view sessions.
