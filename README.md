# remote_viewer

High-performance, low-latency remote desktop for Windows first (then Linux/macOS): connect by
**ID + password**, view the remote screen, control mouse/keyboard, transfer files, and chat.

Clean-room implementation of the *concept* behind tools like AnyDesk / RustDesk / UltraViewer —
**not** affiliated with those products; do not copy their branding or proprietary assets.

Internal Rust crates use the historical `vd-*` prefix.

## Goals

1. Ultra-low latency (LAN glass-to-glass p50 ≤ 40 ms — *initial target*)
2. Usable on weak networks (1 Mbps / 150 ms RTT / 3–5% loss)
3. Lightweight CPU/RAM
4. E2E encrypted, consent-first, **no stealth mode**
5. NAT traversal with relay fallback

## Architecture (one-pager)

Dedicated OS threads for capture→encode→send and recv→decode→render; `tokio` only at the
network edge. QUIC (`quinn`) behind a `Transport` trait; serde+postcard protocol; Noise E2E
crypto; closed-loop `vd-rate` adaptation. Details: [docs/01_ARCHITECTURE.md](docs/01_ARCHITECTURE.md).

## Quickstart

```bash
# full local gate
cargo xtask check

# or piecemeal
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

cargo run -p vd-agent
cargo run -p vd-viewer
```

Dev setup: [docs/12_DEV_SETUP.md](docs/12_DEV_SETUP.md).

## Repo map

| Path | Purpose |
|------|---------|
| `crates/` | Libraries (`vd-proto`, `vd-net`, `vd-capture`, …) |
| `apps/` | `vd-agent`, `vd-viewer`, `vd-signal`, `vd-relay` |
| `tools/` | `vd-bench`, `xtask` |
| `docs/` | Architecture, protocol, tasks, ADRs, agent roles |
| `.cursor/rules/` | Cursor agent rules |

## Multi-agent development

Read **[AGENTS.md](AGENTS.md)** before any work. Task board: [docs/07_TASKS.md](docs/07_TASKS.md).

## Safety / ethics

The host always shows a **visible indicator** while being viewed or controlled and can end the
session at any time. Unattended access requires **explicit opt-in**. This project will never
add hidden/stealth remote-access malware capabilities.

## License

MIT OR Apache-2.0 (see workspace package metadata).
