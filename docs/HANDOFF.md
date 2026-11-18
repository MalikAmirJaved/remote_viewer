# HANDOFF log — remote_viewer

Append-only. Newest entries at the **bottom**. Do not rewrite others' entries.

---

## OPEN QUESTIONS

### Q1 — Noise vs QUIC TLS layering
- **Context:** ADR-0004 requires app-level E2E Noise; quinn also brings TLS.
- **Default (reversible):** Run **Noise for E2E confidentiality** over QUIC streams/datagrams;
  treat QUIC TLS as transport hardening only (or disable cert verification only on
  localhost/LAN demos — never in production internet path without pinning story).
- **Owner for decision refinement:** security-crypto + network-transport in P3.

### Q2 — Product display name vs `vd-*` crates
- **Context:** Product is `remote_viewer`; crates remain `vd-*` for less churn.
- **Default:** Keep `vd-*` until a rename ADR if desired.

### Q3 — rust-toolchain pin
- **Context:** Pinned to `1.99.1` to match the initializer machine; MSRV marked `1.85` in workspace.
- **Default:** Keep pin; bump via PR when CI images move.

---

## 2026-11-17 16:30 · cursor-init · role: architect · T-0001
- Did: Initialized git repo, Cargo workspace, all crate/app stubs, real `vd-common` /
  `vd-proto` / `vd-netsim` / `vd-rate`, full docs/ADRs/roles/Cursor rules/CI/netem scripts,
  seeded task board (P0 marked DONE).
- State: Workspace compiles (`cargo check --workspace`). Product features are stubs returning
  `Error::NotImplemented`. No glass-to-glass video yet.
- Decisions: ADRs 0001–0006 accepted; product name **remote_viewer**; crate prefix `vd-*` retained.
- Next: Parallel wave — T-0101 (capture-codec fake frames), T-0201 (net Transport+netsim),
  T-0402 can wait until P4 but T-0005 skeleton is ready; T-0305 Noise when nearing P3;
  prefer T-0101 + T-0107 + T-0201 + T-0205 first.
- Gotchas: Do not add `todo!()` on tested paths. Hot paths must stay sync. Ethics rules are hard.

## 2026-11-18 11:56 · opencode-a1 · role: architect · T-0108
- Did: Added `openh264` 0.9.10, `eframe`/`egui` 0.37, `windows` 0.62 to
  `[workspace.dependencies]` and wired them into `vd-codec` / `vd-viewer` /
  `vd-capture` (cfg(windows)) so the graph really compiles. Added `tools/loopback`
  member stub (lib + bin) as the T-0104 host. Wrote ADR-0007.
- State: `cargo check --workspace --all-targets` green **including the openh264 C
  build** (`openh264_encoder.lib`/`openh264_common.lib` produced by `cc` with MSVC
  14.44). fmt / clippy `-D warnings` / `cargo test --workspace` green (13 tests).
  `vd-loopback` runs and honestly prints NotImplemented for the pipeline.
- Decisions: ADR-0007 — `openh264` crate with default `source` feature (vendored BSD
  C source, no runtime download, no opaque blob) over `shiguredo_openh264`. A C
  toolchain is now a hard prerequisite (MSVC Build Tools / build-essential); recorded
  in `docs/12_DEV_SETUP.md`. eframe uses `default-features = false` +
  `["default_fonts", "glow"]` — `wgpu` stays the P5/zero-copy choice (T-0502), not MVP.
  T-0104 placement settled: `tools/loopback`, not the product apps (that is T-0208).
- Next: T-0101 (fake capturer) and T-0201 (in-memory Transport over SimLink) can run
  in parallel; then T-0102 encode → T-0103 decode → T-0104 pipeline → T-0105/T-0106.
- Gotchas: `cargo nextest` is not installed on this machine; `cargo test --workspace`
  is the DoD fallback. Adding eframe makes first builds multi-minute on a cold cache.
  openh264 quality on low-latency settings is still unmeasured (T-0102).

## 2026-11-18 12:04 · opencode-a1 · role: capture-codec · T-0101
- Did: `crates/vd-capture/src/fake.rs` — `FakeCapturer` + `FakeCaptureConfig` +
  `render_test_pattern`. Animated BGRA pattern (blue ramp, green ramp, moving red bar,
  low byte of `frame_index` in the top row's alpha) so later stages can prove frame
  identity end-to-end.
- State: 5 unit tests green (config validation, fps→interval, layout, animation/index
  encoding, pacing+dirty-skip). `capture_frame` returns `Ok(None)` before the next
  frame is due instead of blocking — exercises the loopback dirty-skip path.
  Workspace fmt/clippy/tests green.
- Decisions: available in **normal** builds, not `cfg(test)` only — `tools/loopback`
  and `vd-bench` need it. Pacing tests use a `#[cfg(test)]` manual clock setter, so no
  sleeps and no flaky timing assertions.
- Next: T-0102 wires openh264 against this source; T-0104 consumes it in `tools/loopback`.
- Gotchas: `fps` is `u8` (max 255) — a test literal of 1000 does not compile. Pattern
  generation is a per-pixel CPU fill; fine for P1, not a steady-state path to copy.
