# Network adaptation — weak-network strategy

This is the **heart of the product**. All agents working on `vd-rate`, `vd-net`, or codecs
must keep these rules. Targets are *(initial, to be validated)* — see
[05_PERFORMANCE_BUDGET.md](05_PERFORMANCE_BUDGET.md).

## 1. Latest-frame-wins

- Never queue stale video.
- Sender keeps ≤ 1–2 frames in flight; if the pacer/encoder is behind, **drop older** and
  encode the newest capture.
- Receiver jitter buffer is ~0–1 frame: render ASAP, no smoothing delay in v1.

## 2. Skip static frames

- Use capture dirty-rects; if nothing changed, **send nothing** (keepalive/heartbeat only).
- Huge bandwidth win on desktop/text workloads.

## 3. Closed-loop rate control (`vd-rate`)

- Viewer sends `ReceiverReport` every ~50 ms (implemented as Feedback channel ~20 Hz):  
  `recv_bps`, `loss_ppm`, `jitter_us`, `owd_gradient`, `decode_ms`, `last_frame_id`, `need_idr`.
- Agent runs a GCC/BBR-like estimator → target bitrate → drives **encoder bitrate,
  resolution scale, fps cap, and QP** together.
- Skeleton state machine + quality ladder ship in P0; estimator filled in P4.

## 4. Quality ladder

Example ladder (tune with data):

| Step | Resolution | FPS | Nominal bps |
|------|------------|-----|-------------|
| 0 | 1080p | 30 | 4.0 Mbps |
| 1 | 1080p | 20 | 2.5 Mbps |
| 2 | 720p | 30 | 1.5 Mbps |
| 3 | 720p | 15 | 0.8 Mbps |
| 4 | 540p | 15 | 0.5 Mbps |
| 5 | 360p | 10 | 0.25 Mbps |

Modes:

- **text-sharp**: prefer resolution over fps when screen is mostly static/text.
- **motion**: prefer fps for video/games content.

Hysteresis: **fast down** (~100 ms), **slow up** (~2–3 s) to avoid oscillation.

## 5. Loss resilience

- MTU-safe fragments (~1150 B) with `frame_id` / `frag_idx` / `frag_total`.
  Implementation: `vd-net::video::fragment_frame` (`MTU` 1200, data cap 1150 B).
- **NACK** for small gaps.
- **On-demand IDR / intra-refresh** instead of waiting to retransmit large frames.
- Optional XOR / Reed-Solomon FEC for keyframes; adaptive FEC ratio from measured loss
  (`reed-solomon-erasure` later).

## 6. Pacing

- Token-bucket pacer avoids bursts that overflow weak-link buffers (bufferbloat).
- Pacer bitrate follows `vd-rate` target.

## 7. Input priority

- Keyboard / click / wheel: dedicated **high-priority reliable** stream.
- Mouse-move: unreliable datagrams, coalesce to latest, ≤ 125 Hz.
- Input path must never wait behind video.

## 8. Client-side cursor

- Cursor shape/position separate from video; viewer draws cursor locally for instant feel.

## 9. Reconnect

- Transport migration/reconnect keeps session state where possible; resume with IDR.
- Always call input `release_all()` on path loss.

## 10. Relay vs direct

- Continuously measure both paths; prefer direct; fall back to relay seamlessly.
- Relay forwards **opaque encrypted** packets only.

## Test strategy

- Deterministic: `vd-netsim` with seeded RNG.
- OS-level: `scripts/netem/*` and clumsy profiles.
- See [09_TESTING_STRATEGY.md](09_TESTING_STRATEGY.md).
