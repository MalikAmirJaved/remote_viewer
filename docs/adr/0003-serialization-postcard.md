# 0003 — Serialization: serde + postcard

- **Status:** Accepted
- **Date:** 2026-11-17
- **Deciders:** cursor-init (architect)

## Context

Wire messages must be compact (weak networks), versioned, and easy to fuzz/proptest.

## Decision

- `serde` + **`postcard`** for envelope and payloads.
- Versioned `Envelope { version, channel, seq, ts_us, payload }`.
- Explicit protocol version negotiation on Control/Hello; `PROTOCOL_VERSION` in `vd-proto`.
- Breaking changes bump version, update [02_PROTOCOL.md](../02_PROTOCOL.md), add proptest roundtrip.

## Consequences

### Positive

- Small encodings; no schema server; works offline in tests.

### Negative / risks

- Harder for non-Rust clients (acceptable for v1; web viewer is a non-goal).
- Need discipline on enum evolution (use explicit tags, avoid silent reorder).

### Follow-ups

- Fuzz parsers in P8.

## Alternatives considered

1. **protobuf / flatbuffers** — heavier toolchain; better polyglot story (not needed v1).
2. **bincode** — less portable / versioning story than postcard for this use.
