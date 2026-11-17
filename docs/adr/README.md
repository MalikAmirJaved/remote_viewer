# Architecture Decision Records (ADRs)

ADRs capture **significant, durable** technical choices for **remote_viewer**.
They are append-only: never delete or silently rewrite; supersede with a new ADR.

## When an ADR is required

- Changing any item in the [FIXED TECHNICAL DIRECTION](../00_PROJECT_BRIEF.md#fixed-technical-direction)
- Adding a heavy dependency (codec SDK, UI framework, transport stack)
- Changing the wire protocol (also bump `PROTOCOL_VERSION`)
- Changing crate boundaries or the threading model
- Security model changes (crypto, consent, auth)

## Process

1. Copy [0000-template.md](0000-template.md) → `NNNN-short-title.md`
2. Fill Context / Decision / Consequences; link related docs
3. Announce in [HANDOFF.md](../HANDOFF.md); update this index
4. Only the `architect` role (or an explicitly assigned task) merges ADR PRs that alter FIXED DIRECTION

## Index

| ID | Title | Status |
|----|-------|--------|
| [0000](0000-template.md) | Template | — |
| [0001](0001-rust-workspace-and-threading-model.md) | Rust workspace & threading model | Accepted |
| [0002](0002-transport-quic-over-udp.md) | Transport: QUIC over UDP (`quinn`) | Accepted |
| [0003](0003-serialization-postcard.md) | Serialization: serde + postcard | Accepted |
| [0004](0004-e2e-crypto-noise.md) | E2E crypto: Noise XX | Accepted |
| [0005](0005-capture-encode-decode-traits.md) | Capture / encode / decode traits | Accepted |
| [0006](0006-viewer-ui-egui.md) | Viewer UI: egui/eframe | Accepted |
