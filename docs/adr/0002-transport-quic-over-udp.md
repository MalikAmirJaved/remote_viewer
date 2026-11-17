# 0002 — Transport: QUIC over UDP (`quinn`)

- **Status:** Accepted
- **Date:** 2026-11-17
- **Deciders:** cursor-init (architect)

## Context

We need multiplexed reliable streams (control, input, files) and unreliable datagrams
(video, audio, mouse-move) with NAT traversal and relay fallback.

## Decision

- Custom application protocol on **QUIC** via **`quinn`**.
- Reliable QUIC streams for Control / Input / Cursor / Clipboard / File / Chat.
- **Unreliable datagrams** for Video / Audio / InputMove / Feedback.
- Behind a `Transport` trait in `vd-net` so we can swap implementations.
- MTU baseline **1200** bytes; fragment video to ~1150 B payloads.
- Hole-punch from the same UDP socket used by QUIC; relay forwards opaque encrypted packets.

## Consequences

### Positive

- One port/socket story; stream priorities possible; TLS-like security layering with our E2E Noise on top of (or instead of relying solely on) QUIC TLS — see ADR-0004 for app-level E2E.

### Negative / risks

- Quinn datagram congestion may interact poorly with custom `vd-rate` control on lossy links.
- QUIC stack complexity vs raw UDP.

### Follow-ups

- Early benchmark vs `str0m` WebRTC if pacing fights us; swap allowed by this ADR’s trait boundary.
- Tasks under P2/P3 for quinn wiring.

## Alternatives considered

1. **WebRTC (`str0m`) first** — deferred as alternate behind the same trait.
2. **Raw UDP + custom reliability** — more code, reinvent congestion; keep as last resort.
