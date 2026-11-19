# Changelog

All notable changes to **remote_viewer** are documented here.

Format inspired by [Keep a Changelog](https://keepachangelog.com/). Versions follow SemVer once
releases begin.

## [Unreleased]

### Added

- P0 foundation: Cargo workspace, toolchain/lint/deny configs, CI workflow
- Library crates with trait stubs (`vd-capture`, `vd-codec`, `vd-net`, `vd-input`, `vd-crypto`, …)
- Real initial implementations: `vd-common` clock/tracing, `vd-proto` + proptest, `vd-netsim`, `vd-rate` ladder
- App/tool stubs: `vd-agent`, `vd-viewer`, `vd-signal`, `vd-relay`, `vd-bench`, `xtask`
- Documentation set (`docs/00`–`12`), ADRs 0001–0006, agent roles, Cursor rules
- Task board seeded through P6 coarse items; multi-agent workflow
- P1 pipeline: deterministic fake capturer, openh264 software H.264 encoder/decoder
  (ADR-0007), and a four-thread capture→encode→decode→present loopback harness
  (`tools/loopback`) with per-stage timings and drop-oldest queues
- Real Windows capture: DXGI Desktop Duplication backend with dirty rects and
  access-lost recovery (`vd-capture::dxgi`)
- P2 groundwork: in-memory `Transport` adapter over `vd-netsim` with contract tests

## [0.1.0] — 2026-11-17

- Initial repository scaffold (`chore: initialize … [T-0001]`)
