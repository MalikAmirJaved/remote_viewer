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

## [0.1.0] — 2026-11-17

- Initial repository scaffold (`chore: initialize … [T-0001]`)
