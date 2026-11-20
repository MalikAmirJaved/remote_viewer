# Project status — remote_viewer

**Updated:** 2026-11-20 by cursor-grok (network-transport, T-0207)

## Current phase

**P1 Local pipeline — implementation complete; 5/8 DONE and 3 in REVIEW** (T-0105
viewer window, T-0106 latency report, T-0107 DXGI capturer). No P1 work is unimplemented.
**P2 in progress**: T-0201 DONE. In REVIEW: T-0202…T-0207 (QUIC through pacer),
T-0209/T-0210 (input). Next on the LAN path is **T-0208** (agent↔viewer demo flags)
once T-0203 + T-0206 are accepted. `apps/vd-signal/**` stays locked for T-0301 until
2026-11-20 10:42 UTC.

## Task completion by phase

| Phase | Done | Total tracked | % |
|-------|------|---------------|---|
| P0 | 8 | 8 | 100% |
| P1 | 5 | 8 | 62.5% |
| P2 | 1 | 10 | 10% |
| P3 | 0 | 9 | 0% |
| P4 | 0 | 10 | 0% |
| P5 | 0 | 3 | 0% |
| P6 | 0 | 6 | 0% |
| **All** | **14** | **54** | **~26%** |

*(P7–P9 tracked in roadmap; fine-grained tasks to be added later.)*

## Active locks

`apps/vd-signal/**` — T-0301, opencode-a2, until 2026-11-20 10:42 UTC. See
[agents/LOCKS.md](agents/LOCKS.md).

## Last HANDOFF headlines

1. `2026-11-17 · cursor-init · architect · T-0001` — workspace/docs/stubs initialized
2. `2026-11-18 · opencode-a1 · … · T-0108 → T-0104` — openh264 deps/pins, fake capture,
   SW encode/decode, loopback threads; all measured on Windows
3. `2026-11-19 · opencode-a2 · capture-codec · T-0107` — real DXGI capturer; capture
   p50 3.1 ms (dev) / 3.4 ms (release) at 1680×1050
4. `2026-11-19 · opencode-a2 · reviewer` — T-0101…T-0104, T-0108, T-0201 accepted as
   DONE; P1 unblocked for T-0105/T-0106
5. `2026-11-19 · opencode-a2 · viewer-ui · T-0105` — real eframe window presents
   decoded loopback frames; release: 28.9 fps, 0 dropped at 960×540; clean close
6. `2026-11-19 · opencode-a2 · qa-bench · T-0106` — `vd-bench` reports per-stage
   p50/p95/p99; 720p30 total p50 24.75 ms with 0 drops (software H.264)
7. `2026-11-19 · opencode-a2 · network-transport · T-0202` — real QUIC connect/accept
   with pinned dev cert; two-process loopback ping/pong, idle timeout configurable
8. `2026-11-19 · opencode-a2 · network-transport · T-0205` — video fragmenter: encoded
   frame → 1150 B `VideoFrameFragment`s, validated, tested against a real `EncodedFrame`
9. `2026-11-20 · cursor-grok · network-transport · T-0203` — Control and Input on
   separate QUIC streams; Input send priority 2, Control 1
10. `2026-11-20 · cursor-grok · network-transport · T-0204` — tagged QUIC datagrams
    for Video/Feedback/InputMove/Audio; 1150 B soft cap
11. `2026-11-20 · cursor-grok · network-transport · T-0206` — latest-frame-wins
    video reassembler; netsim loss/reorder tests
12. `2026-11-20 · cursor-grok · network-transport · T-0207` — token-bucket pacer
    enforces target bps (±10% in unit test)

## Open questions

See [HANDOFF.md](HANDOFF.md) `## OPEN QUESTIONS`.

## Known blockers

- Reviews needed: **T-0105 / T-0106 / T-0107** (P1) and **T-0202…T-0207 / T-0209 /
  T-0210** (P2), plus **T-0301** (P3).
- `cargo clippy --workspace --all-targets -D warnings` fails in `vd-input` (T-0209/T-0210):
  `MoveCoalescer::feed` is public while `DesktopBounds` and `HostInput` are `pub(crate)`,
  plus unused items and `must_use` on a unit `reset`. `vd-net` itself is clean.
- Accepted P1 limits (see HANDOFF 2026-11-19): DXGI rotated outputs and the hardware
  cursor are not captured; one `Vec<u8>` allocation per captured frame until the P5
  buffer pool / GPU path; the viewer converts RGBA→`ColorImage` and re-uploads the full
  texture per frame on the UI thread until the P5 GPU path; `vd-bench` percentiles cover
  presented frames only and `present` is still the loopback stand-in.
- QUIC identity is a per-process self-signed certificate pinned out of band; real
  identity distribution is P3 signalling, not a P2 deliverable.
- Video fragments carry no dirty rects yet: `vd_codec::EncodedFrame` does not propagate
  capture dirty rects, so `dirty_rects` is `None` until a codec/proto change (architect).

## Latest benchmark numbers

| Date | Profile | p50 ms | p95 ms | Notes |
|------|---------|--------|--------|-------|
| — | — | — | — | **no glass-to-glass measurement yet** |

Stage-level numbers (capture) are in
[05_PERFORMANCE_BUDGET.md](05_PERFORMANCE_BUDGET.md#stage-measurements-host-side-not-glass-to-glass).

