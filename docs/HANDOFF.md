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

## 2026-11-18 12:38 · opencode-a1 · role: capture-codec · T-0103
- Did: `OpenH264Decoder` is real — lazy init on first decode, persistent reference
  state, RGBA8 output via `DecodedYUV::write_rgba8`. Three tests: keyframe roundtrip,
  five-frame delta sequence decoded in order, empty/garbage input handling.
- State: measured roundtrip error on the 64x48 test pattern is **2.44/255 mean
  absolute RGB difference** (budget 12.0), alpha fully opaque. Workspace 33 tests,
  fmt/clippy green.
- Decisions: output is **RGBA8, not BGRA**. Acceptance says "BGRA/YUV"; OpenH264
  exposes only `write_rgb8`/`write_rgba8`, and the viewer's texture upload wants RGBA,
  so BGRA would add a full per-frame swap pass with no consumer. Flagged for the
  reviewer rather than silently reworded. Decoder is created on first `decode()` so a
  configured-but-unused decoder allocates nothing.
- Next: T-0104 can now build the loopback pipeline end-to-end (fake → encode → decode →
  timings) and is unblocked.
- Gotchas: P frames **must** be decoded in order against the same decoder instance —
  there is no random access without an IDR. `decode()` returning `Ok(None)` from
  OpenH264 (no frame yet) is surfaced as `Error::Unavailable`, so callers must not
  treat every access unit as producing exactly one frame. Roundtrip error scales with
  bitrate; the 12.0 budget is tuned for 500 kbps at 64x48.

## 2026-11-18 12:44 · opencode-a1 · role: architect · T-0104
- Did: `tools/loopback/src/lib.rs` now runs four dedicated OS threads (capture, encode,
  decode, present) joined by bounded `crossbeam-channel` queues (`queue_cap` 1–2).
  `LatestSender::send_latest` evicts the oldest queued item instead of blocking, so
  capture cadence never depends on encode. `RunStats` reports per-stage timing sums,
  mean/max frame total and per-queue drop counters. CLI gained `--frames` and
  `--queue-cap` and prints the summary.
- State: **measured on this machine**, 320x180, 30 frames at 30 fps:
  `captured=30 encoded=30 decoded=30 presented=30 dropped=0`, capture 110 ms total,
  encode 273 ms, decode 122 ms, mean frame total 16.8 ms, max 33.2 ms. Workspace 37
  tests, fmt/clippy green. 12-frame unit run at 64x48 has zero drops and last index 11.
- Decisions: all three in-process queues share one `queue_cap` because the pacer/network
  stage is bypassed here; production wiring keeps the architecture's per-link caps.
  `LatestReceiver`'s `Drop` sets a flag the sender checks — without it the sender's own
  receiver clone keeps the channel "connected", so a dead consumer would spin the
  producer forever. "Present" is a dimension-validated stand-in until T-0105, so its
  measured cost (~0) is **not** representative of texture upload.
- Open items (honest): OpenH264 logs two advisory warnings at init —
  `bEnableFrameSkip = 0` means the CBR bitrate is advisory rather than enforced, and
  screen-change detection is off (deliberate, keeps keyframes on-demand). Rate
  enforcement belongs to T-0403; do not claim bitrate accuracy until it exists.
- Next: T-0105 can present `DecodedFrame` (RGBA8) in egui; T-0106 can consume `RunStats`
  for the latency report. T-0107 swaps `FakeCapturer` for DXGI without touching stages.
- Gotchas: capture polls at 1 ms while `capture_frame` returns `Ok(None)` — harness-only,
  production DXGI blocks. Entry indexing is 0-based and `force_idr` fires only on index 0,
  so a decoder joining mid-stream needs an explicit IDR. Per-stage totals only cover
  **presented** frames; dropped frames contribute no timing.

## 2026-11-19 09:07 · opencode-a2 · role: capture-codec · T-0107
- Did: `crates/vd-capture/src/dxgi.rs` — real DXGI Desktop Duplication capturer: D3D11
  device with BGRA support, monitor selection on the device's adapter, non-blocking
  `AcquireNextFrame(0)`, staging-texture readback into tightly packed BGRA8, dirty rects
  from frame metadata, and `DXGI_ERROR_ACCESS_LOST` recovery that rebuilds the
  duplication + staging texture and raises a keyframe hint. Added the crate-local
  `windows` features (`Win32_Graphics_Direct3D`, `..._Direct3D11`) and a default
  `Capturer::take_needs_keyframe()` trait method. Replaced the Windows stub with a
  re-export; Linux/macOS stubs untouched.
- State: measured on this machine, 1680x1050, with desktop activity: `dev` build 60
  frames — mean 3225 us, p50 3121 us, max 4523 us; `--release` 53 frames — mean
  3470 us, p50 3388 us, max 5623 us. Release showed no gain, so this stage is bound by
  the GPU copy + `Map` synchronisation, not the CPU row copy or the per-frame
  allocation. p50 is inside the ≤4 ms capture budget in both runs; the tail is not.
  Verified by the ignored test
  `vd_capture::dxgi::tests::dxgi_captures_primary_output_and_recovers` — start, frame
  acquisition, BGRA size/format, dirty-rect bounds, rebuild path, clean stop. Workspace
  fmt / clippy `-D warnings` / `cargo test --workspace` green (41 passed, 1 ignored).
- Decisions: keyframe signalling is a default trait method (`take_needs_keyframe`)
  instead of a field on `CapturedFrame`, so the trait stays the contract and fake/stub
  backends need no change. Dirty rects are best-effort: an empty vector means
  "whole frame", so metadata failure degrades to a full encode. Rotated outputs are
  rejected with `Unavailable` (T-0605) rather than emitting a sideways frame.
  `capture_ts_us` is the CPU-copy time, not the DXGI desktop present time.
- Honest gaps: real `ACCESS_LOST` was **not** triggered (needs desktop lock, switch or
  resolution change); only the rebuild routine it calls is exercised live. Hardware
  cursor is not captured (T-0409), secondary-adapter outputs are not enumerated
  (T-0605), and every delivered frame allocates one `Vec<u8>` because `CapturedFrame`
  has no buffer-return path — this violates the "no per-frame alloc" goal until the P5
  pool/GPU path. Only the Windows target was compiled here.
- Next: P1 close-out — T-0105 (egui presents `DecodedFrame`) and T-0106 (per-stage
  report). Swapping `FakeCapturer` for `DxgiCapturer` in `tools/loopback` needs its own
  task because that path belongs to T-0104; T-0405 must consume
  `take_needs_keyframe` when it drives the encoder.
- Gotchas: the capturer is thread-affine (COM apartment, D3D11 immediate context,
  duplication object) — create/capture/drop on one OS thread. `WAIT_TIMEOUT` on a
  static desktop is normal, not an error (correcting the T-0104 note above: the DXGI
  backend polls with a zero timeout instead of blocking). Every successful acquire must
  release the frame exactly once, including when the metadata says pointer-only.

## 2026-11-19 09:20 · opencode-a2 · role: reviewer · T-0101…T-0104, T-0108, T-0201
- Did: reviewed all six REVIEW tasks (author opencode-a1): read the diffs, matched every
  acceptance item against the code, and re-ran the DoD on this checkout — `fmt --check`,
  `clippy --workspace --all-targets -D warnings`, `cargo test --workspace` (41 passed,
  1 ignored real-desktop test). Marked all six `DONE`, added a per-task **Review** note,
  and updated `STATUS.md` + `CHANGELOG.md`.
- State: P1 6/8 — T-0105 (egui present) and T-0106 (latency report) remain. P2 1/10 —
  T-0202 (quinn wrapper) is now claimable. No active locks. T-0107 is still `REVIEW`.
- Decisions: T-0103 accepted with **RGBA8** rather than the literal "BGRA/YUV" wording —
  that is what OpenH264 exposes and what the egui texture upload needs; the deviation is
  recorded in the task note instead of silently reworded. T-0104's overload evidence
  accepted at the channel-primitive level (100 pushes into cap 1/2), because that is
  exactly where the drop-oldest policy lives. T-0201's adapter staying in `tests/` is
  correct per architecture §3.
- Next: claim T-0105 (viewer-ui, `apps/vd-viewer`) and/or T-0106 (qa-bench,
  `tools/vd-bench` + perf doc); T-0202 is also unblocked. T-0107 needs a reviewer before
  its DXGI path is considered accepted.
- Gotchas: same-tool review (opencode-a2 reviewed opencode-a1) is weaker than an
  independent reviewer; if a second contributor joins, re-check T-0103's format
  deviation and T-0107's never-triggered `ACCESS_LOST` path first. `decode()` returning
  `Unavailable` for access units that produce no frame is a real P2 tolerance item.


