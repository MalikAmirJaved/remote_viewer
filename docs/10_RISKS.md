# Risk register — remote_viewer

| ID | Risk | Impact | Likelihood | Mitigation |
|----|------|--------|------------|------------|
| R1 | HW encoder integration slips (P5) | High | High | Ship SW openh264 path first (P1); trait boundary |
| R2 | Quinn datagram congestion fights `vd-rate` | High | Med | Early netsim benches; ADR-0002 allows `str0m` swap |
| R3 | UAC / secure desktop complexity (P7) | High | High | Isolate service+helper; strict consent/indicator |
| R4 | Weak-net targets missed | High | Med | P4 dedicated; netsim traces; quality ladder hysteresis |
| R5 | Multi-agent merge conflicts | Med | High | LOCKS + Touches + worktrees + architect owns proto |
| R6 | Crypto layering confusion (Noise vs QUIC TLS) | High | Med | ADR-0004; security-crypto role owns design |
| R7 | Accidental stealth-like features | Critical | Low | Ethics rules in every doc + reviewer checklist |
| R8 | Platform capture access-lost loops | Med | High | Re-init + IDR; capture-codec runbook |
| R9 | Relay cost / abuse | Med | Med | Caps, auth, rate limits (P3/P8) |
| R10 | License / codec SDK redistribution | Med | Med | Prefer system/HW APIs; document redistribution in P9 |

## Related

[06_ROADMAP.md](06_ROADMAP.md) · [04_SECURITY.md](04_SECURITY.md)
