# Performance budget — remote_viewer

All numbers labeled *(initial target, to be validated)* until measured with the harness.

## Latency goals

| Scenario | Glass-to-glass p50 | Glass-to-glass p95 |
|----------|--------------------|--------------------|
| LAN | ≤ 40 ms | ≤ 60 ms |
| WAN-good (RTT ≤ 40 ms) | ≤ 90 ms | ≤ 130 ms |

Per-stage budgets: [01_ARCHITECTURE.md](01_ARCHITECTURE.md#4-per-stage-latency-budget-initial-targets-to-be-validated).

## Resource goals

| Metric | Target |
|--------|--------|
| Agent idle CPU | < 1 % |
| Agent active (HW encode) | < 10 % of one core |
| Agent RAM | < 150 MB |
| Weak net usability | ≥ 15 fps, readable text, responsive input @ 1 Mbps / 150 ms RTT / 3–5 % loss |

## Network profiles

| Profile | RTT | Loss | BW | Target fps | Max added latency* | Min readability |
|---------|-----|------|----|------------|--------------------|-----------------|
| `lan` | ~0 | 0 % | ∞ | 30 | ≤ 40 ms g2g p50 | crisp 1080p |
| `wan-good` | 40 ms | 0.1 % | 20 Mbps | 25–30 | ≤ 90 ms g2g p50 | clear text |
| `wan-weak` | 150 ms | 3 % | 1 Mbps | ≥ 15 | degrade, no freeze | readable @ ≥ 540p |
| `mobile-3g` | 300 ms | 5 % | 400 kbps | ≥ 10 | best-effort | readable @ 360p |
| `lossy-wifi` | 30 ms + 40 ms jitter | 8 % burst | 5 Mbps | ≥ 15 | controlled via FEC/IDR | text mostly OK |

\*Added latency beyond one-way network delay should stay within stage budgets; g2g includes network.

Presets: [`scripts/netem/`](../scripts/netem) · in-process: `vd-netsim`.

## How we measure latency

1. Embed `frame_id` + `capture_ts_us` in video fragments (already in proto).
2. Viewer records `render_ts_us` (monotonic).
3. For absolute g2g across hosts: NTP-ish offset via Control Ping/Pong RTT/2 estimate
   *(initial method)*; later PTP-like refinement if needed.
4. Report **p50 / p95 / p99** per profile in the results table below.
5. Overlay optional on-screen frame counter for manual camera tests.

## CI perf gate

- Criterion benches for proto encode/decode, rate controller, netsim throughput.
- Gate: **no regression > 10 %** vs baseline committed in this file’s results table
  (once baselines exist).
- Missing GPU / HW encode: mark HANDOFF honestly; do not invent numbers.

## Results table

| Date | Profile | p50 ms | p95 ms | p99 ms | fps | Agent CPU % | Notes |
|------|---------|--------|--------|--------|-----|-------------|-------|
| — | — | — | — | — | — | — | **not measured yet** |

### Stage measurements (host-side, not glass-to-glass)

| Date | Stage | Context | N | p50 | max | Notes |
|------|-------|---------|---|-----|-----|-------|
| 2026-11-19 | Capture — DXGI Desktop Duplication, 1680×1050 BGRA8 | dev box, `dev` build, moving window | 60 | 3.1 ms | 4.5 ms | `capture_frame` incl. staging copy + tight BGRA buffer. p50 inside the ≤ 4 ms capture budget; max over it. |
| 2026-11-19 | Capture — same path, `--release` | 53 | 3.4 ms | 5.6 ms | No gain over the debug run ⇒ the cost is dominated by the GPU copy + `Map` synchronisation, not the CPU row copy or the per-frame allocation. Still inside budget at p50; the tail is not. |

### Loopback pipeline — `vd-bench` (host-side, single machine, software H.264)

Per-stage cost of the four-thread loopback pipeline (`tools/loopback`), reported by
`tools/vd-bench` as nearest-rank percentiles. This is **not** glass-to-glass: no
network, no viewer texture upload, no camera. `present` is the loopback stand-in
(frame validation), so its 0.00 ms is expected.

Reproduce (release build, this machine):

```
cargo run --release -p vd-bench -- --width 1280 --height 720 --fps 30 --frames 300
cargo run --release -p vd-bench -- --width 1280 --height 720 --fps 60 --frames 300 --bitrate 6000000
cargo run --release -p vd-bench -- --width 1920 --height 1080 --fps 30 --frames 300 --bitrate 8000000
```

| Date | Config | N | capture p50 | encode p50 | decode p50 | total p50 | total p95 | dropped | Notes |
|------|--------|---|-------------|------------|------------|-----------|-----------|---------|-------|
| 2026-11-19 | 1280×720 @ 30 fps | 300 | 5.55 ms | 16.05 ms | 2.71 ms | 24.75 ms | 27.83 ms | 0 | Capture is the **fake** source (pattern generation + allocation), so its 5.55 ms is not comparable to the DXGI 3.4 ms above. Software encode is ~2× the ≤ 8 ms encode budget — that budget assumes hardware encode (P5). |
| 2026-11-19 | 1280×720 @ 60 fps | 292 | 5.52 ms | 16.71 ms | 2.93 ms | 25.31 ms | 30.18 ms | 8 (capture→encode) | Encode p50 exceeds the 16.7 ms frame interval, so 8 of 300 frames were evicted between capture and encode. Max single-frame total 80.32 ms (encoder tail). |
| 2026-11-19 | 1920×1080 @ 30 fps | 274 | 12.27 ms | 36.61 ms | 6.45 ms | 55.61 ms | 61.47 ms | 26 (capture→encode) | Software encode cannot hold 1080p30 on this CPU: 26 frames evicted. Decode p50 6.45 ms is just over the ≤ 6 ms decode budget. Max total 168.15 ms. |

Reading: the software path holds **720p30 with zero drops** on this machine; 720p60 and
1080p30 lose frames at capture→encode because software H.264 encode (16–37 ms p50)
exceeds the frame interval. That is the expected trigger for the P5 hardware encoder
(T-0501) and rate-control work (T-0403), not a defect in the queue policy — the drops
are latest-frame-wins keeping capture on cadence.

Implementation notes: `RunStats.samples` (added in T-0106) carries one `StageTimings`
per presented frame; percentiles are nearest-rank over those samples (no interpolation),
so a reported p50 is a value that actually occurred. Async entries are excluded because
a dropped frame has no present-side sample.

## Related

[03_NETWORK_ADAPTATION.md](03_NETWORK_ADAPTATION.md) · [09_TESTING_STRATEGY.md](09_TESTING_STRATEGY.md) · tools/`vd-bench`
