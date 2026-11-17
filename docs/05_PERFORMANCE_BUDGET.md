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

## Related

[03_NETWORK_ADAPTATION.md](03_NETWORK_ADAPTATION.md) · [09_TESTING_STRATEGY.md](09_TESTING_STRATEGY.md) · tools/`vd-bench`
