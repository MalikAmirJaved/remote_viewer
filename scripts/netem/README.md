# Netem / clumsy presets

Use these profiles with OS-level shaping for E2E tests. In-process tests should prefer
[`vd-netsim`](../../crates/vd-netsim) for determinism.

| Profile | RTT | Loss | Bandwidth | Notes |
|---------|-----|------|-----------|-------|
| `lan` | ~0 ms | 0 % | unlimited | Baseline |
| `wan-good` | 40 ms | 0.1 % | 20 Mbps | Comfortable broadband |
| `wan-weak` | 150 ms | 3 % | 1 Mbps | Product must stay usable |
| `mobile-3g` | 300 ms | 5 % | 400 kbps | Stress / degrade gracefully |
| `lossy-wifi` | 30 ms + 40 ms jitter | 8 % burst | 5 Mbps | Burst loss |

Scripts:

- Linux: `lan.sh`, `wan-good.sh`, `wan-weak.sh`, `mobile-3g.sh`, `lossy-wifi.sh` — `tc netem` helpers (require root / `NET_ADMIN`).
- Windows: use [clumsy](https://jagt.github.io/clumsy/) with the matching `.txt` notes; automation TBD.

See [docs/05_PERFORMANCE_BUDGET.md](../../docs/05_PERFORMANCE_BUDGET.md) for targets.