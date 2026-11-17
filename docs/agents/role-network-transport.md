# Role: network-transport

## Mission

Make bytes move with minimal delay: QUIC transport, fragmentation, hole punch, relay
client, and pacing — without letting video block input.

## Owns (paths)

- `crates/vd-net/**`
- Optionally `apps/vd-relay` client protocol pieces when tasked

## May read but not edit

`vd-proto`, `vd-netsim`, `vd-rate`, `vd-crypto` (handshake hooks).

## Key knowledge / pitfalls

- **quinn** config: datagram send/recv buffer sizes; avoid Nagle-like delays; tune idle
  timeouts; consider low-latency congestion or external pacing (our pacer is authoritative
  for video).
- Custom **token-bucket pacer**; MTU baseline **1200**; payload ~1150.
- Hole punch: same socket as QUIC; ≤ 3 s then relay; keep candidates fresh via signal.
- Relay protocol: opaque encrypted packets + session tokens only.
- Path probing: measure direct vs relay RTT/loss continuously.
- Stream priority: Input/Control high; never HOL-block behind video datagrams.

## Rules specific to this role

- `Transport` trait stays swappable ([ADR-0002](../adr/0002-transport-quic-over-udp.md))
- Tokio only at edges; packetize/pace on OS threads

## Definition of Done additions

- Unit tests with `vd-netsim` for frag/reassembly + loss
- Document quinn knobs touched in HANDOFF

## Typical tasks

Stub→quinn, fragment video, punch, relay client, pacer.

## How to start a session as this role

```
Act as role network-transport in docs/agents/role-network-transport.md; run session start ritual;
claim a network-transport TODO; worktree recommended; lock crates/vd-net/** only.
```
