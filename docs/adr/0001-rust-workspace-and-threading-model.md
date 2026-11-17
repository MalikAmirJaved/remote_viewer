# 0001 — Rust workspace and threading model

- **Status:** Accepted
- **Date:** 2026-11-17
- **Deciders:** cursor-init (architect)

## Context

remote_viewer must hit glass-to-glass latency budgets while remaining maintainable for
multiple parallel AI agents. Async runtimes simplify networking but add jitter and
hidden allocations if used on per-frame hot paths.

## Decision

- Rust **stable**, edition **2021**, toolchain pinned in `rust-toolchain.toml`.
- Cargo **workspace**, `resolver = "2"`, shared `[workspace.dependencies]` and `[workspace.lints]`.
- **`tokio` ONLY** in network/server crates and app orchestration (`vd-signal`, `vd-relay`, connect paths).
- Capture→encode→send and recv→decode→render run on **dedicated OS threads** with bounded
  `crossbeam-channel`s (capacity 1–2 for video; drop-oldest / latest-frame-wins).
- **No `.await` in hot paths.**
- `#![forbid(unsafe_code)]` everywhere except `vd-capture`, `vd-codec`, `vd-input`.

## Consequences

### Positive

- Predictable latency; clear ownership of threads; agent-friendly crate boundaries.

### Negative / risks

- More explicit thread plumbing than a fully async design.
- Care needed at async↔sync boundaries (use dedicated runtime threads, never block the tokio reactor on encode).

### Follow-ups

- P1 local pipeline tasks wire the channels end-to-end.
- Docs: [01_ARCHITECTURE.md](../01_ARCHITECTURE.md)

## Alternatives considered

1. **Fully async pipeline on tokio** — rejected: scheduler latency and allocation risk on weak machines.
2. **Edition 2024 now** — deferred; bump via future ADR when MSRV allows and value is clear.
