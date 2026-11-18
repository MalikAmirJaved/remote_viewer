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

## 2026-11-18 12:12 · opencode-a1 · role: network-transport · T-0201
- Did: `SimPair` + `SimTransport` implementing the real `Transport` trait over
  `vd_netsim::SimLink`, in `crates/vd-net/tests/common/`; 7 integration tests in
  `tests/sim_transport.rs`.
- State: channel-preserving round trip (all 10 channels, in order), independent
  directions, latency deferral (49 ms no / 50 ms yes), total-loss drop, close
  semantics (`send` → `Error::Io`, peer unaffected), malformed/empty frame →
  `Error::Protocol` (no panics). Workspace fmt/clippy/tests green.
- Decisions: adapter lives in `tests/`, **not** `src/`, and `vd-netsim` is a
  dev-dependency — a lateral feature-crate dependency in the product graph would
  violate docs/01_ARCHITECTURE.md §3. Wire frame is `[channel_tag][payload]`;
  `SendMode` is accepted but not modelled differently because `SimLink` has no
  retransmission/ordering model yet. If another crate needs the adapter later,
  promote to a gated module rather than copying it.
- Next: T-0202 (quinn connect/accept) can start; T-0205 fragmenter is independent,
  and T-0206's reassembler tests can reuse `tests/common`.
- Gotchas: `SimLink` only delivers when *time advances* — tests must tick even for
  the zero-latency `lan` profile (`advance_ms(0)`). `Channel` is `#[repr(u8)]`; the
  tag table in `tests/common` must be updated if a channel is ever inserted.

## 2026-11-18 12:31 · opencode-a1 · role: capture-codec · T-0102
- Did: `OpenH264Encoder` is real — openh264 0.9.10 with screen-content real-time
  usage, CBR bitrate mode, infinite GOP (on-demand IDR only), single internal thread,
  low complexity, `skip_frames(false)`, scene-change/adaptive-quantization/background
  detection off. BGRA→I420 into a reused `YUVBuffer`; `set_bitrate` rebuilds.
- State: 5 tests green — Annex-B start code, first frame IDR/I, compressed output
  (smaller than raw BGRA), P frame on the second frame, `force_idr` keyframe, rebuild
  emits a keyframe and resets the counter, invalid args rejected. Workspace total 30
  tests, fmt/clippy green.
- Decisions: `skip_frames(false)` for P1 so the pipeline stays 1:1 and loopback timing
  means something; rate-driven dropping is T-0403's job. `force_idr` maps directly to
  `Encoder::force_intra_frame`. Bitrate changes are a slow path (encoder rebuild), so
  T-0403 must treat them as rate-decision events, not per-frame calls. No B-frames:
  OpenH264's real-time path does not emit them — that is a property of the encoder,
  not something a test asserts.
- Next: T-0103 decoder roundtrip against this output (encode→decode→compare).
- Gotchas: 64x48 works; odd dimensions are rejected because I420 needs even W/H.
  Encoded output is Annex-B (4-byte start codes), not AVCC — T-0205 fragmentation and
  T-0103 decoding must assume Annex-B, or a conversion step is added later.
