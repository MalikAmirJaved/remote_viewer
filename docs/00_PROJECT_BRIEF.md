# Project brief — remote_viewer

**remote_viewer** is a high-performance, low-latency remote desktop tool (same *concept* as
UltraViewer / AnyDesk / RustDesk): connect to another PC by ID + password, see its screen,
control mouse/keyboard, transfer files, and chat.

This is a **clean-room** implementation. Do **not** use UltraViewer (or other vendors’)
names, logos, assets, or reverse-engineered proprietary protocols.

Internal crate prefix `vd-*` is historical (codename); the product name is **remote_viewer**.

## Goals (priority order)

1. **Ultra-low latency**: glass-to-glass p50 ≤ 40 ms on LAN, ≤ 90 ms on good WAN (RTT ≤ 40 ms). *(initial targets, to be validated)*
2. **Smooth on weak internet**: usable (≥ 15 fps, readable text, responsive input) at 1 Mbps / 150 ms RTT / 3–5 % loss. Degrade gracefully; never freeze; never build queue delay.
3. **Lightweight**: agent idle CPU < 1 %; active streaming < 10 % of one core with HW encode; RAM < 150 MB. *(initial targets)*
4. **Secure by default**: E2E encrypted, explicit consent, no hidden/stealth mode.
5. **Reliable**: auto-reconnect; NAT traversal with relay fallback.
6. **Windows first**, then Linux, then macOS.

## Non-goals (v1)

Mobile apps, web viewer, session recording, enterprise address book/SSO, wake-on-LAN.
Keep architecture open to them (see [01_ARCHITECTURE.md](01_ARCHITECTURE.md) extension points).

## Ethics / safety (hard rules)

- The host **always** shows a **visible indicator** while viewed/controlled; host can **end the session anytime**.
- **Unattended access** requires explicit opt-in by the host owner + permanent password / device approval.
- **No** hidden mode, process-name masking, persistence tricks, or credential harvesting.
- Never add anything that makes the agent function as stealth remote-access malware.

## Fixed technical direction

Recorded as ADRs; revise only via a new ADR. Summary:

| Area | Decision | ADR |
|------|----------|-----|
| Language | Rust stable, edition 2021 | [0001](adr/0001-rust-workspace-and-threading-model.md) |
| Async | tokio only outside hot paths; OS threads + crossbeam for media | [0001](adr/0001-rust-workspace-and-threading-model.md) |
| Transport | QUIC (`quinn`) behind `Transport` | [0002](adr/0002-transport-quic-over-udp.md) |
| Serialization | serde + postcard | [0003](adr/0003-serialization-postcard.md) |
| Crypto | Noise XX + Argon2id password challenge | [0004](adr/0004-e2e-crypto-noise.md) |
| Capture/Codec | Traits; DXGI/openh264/HW path | [0005](adr/0005-capture-encode-decode-traits.md) |
| Viewer UI | egui/eframe MVP | [0006](adr/0006-viewer-ui-egui.md) |

## Related docs

- [01_ARCHITECTURE.md](01_ARCHITECTURE.md) · [02_PROTOCOL.md](02_PROTOCOL.md) · [03_NETWORK_ADAPTATION.md](03_NETWORK_ADAPTATION.md)
- [04_SECURITY.md](04_SECURITY.md) · [05_PERFORMANCE_BUDGET.md](05_PERFORMANCE_BUDGET.md) · [06_ROADMAP.md](06_ROADMAP.md)
- [07_TASKS.md](07_TASKS.md) · [AGENTS.md](../AGENTS.md) · [STATUS.md](STATUS.md)
