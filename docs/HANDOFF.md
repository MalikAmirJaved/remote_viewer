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

## 2026-11-19 04:34 · opencode-a2 · role: viewer-ui · T-0105
- Did: turned `apps/vd-viewer` from a stub into a real eframe window. `loopback.rs` runs
  fake capture → openh264 encode → openh264 decode on one worker thread and publishes
  decoded RGBA frames into a cap-1 latest-frame channel (worker evicts stale frames).
  `main.rs` uploads the newest frame with `TextureHandle::set`, letterboxes it, and adds
  Start/Stop/Restart, Esc=close, F5=restart, a status bar (presented / pipeline /
  dropped / measured UI fps / worker error) and `on_exit` worker shutdown. Added
  `vd-capture` + `crossbeam-channel` to the viewer's dependencies.
- State: verified on this machine at 960x540, 30 fps requested — `--release` window
  reported 265 presented / 265 pipeline / 0 dropped and 28.9 UI fps (screenshot shows
  the fake gradient rendered); `dev` build managed ~8 fps because openh264 is compiled
  with `OPT_LEVEL=0` in dev. Window closed via `WM_CLOSE` with exit code 0 and the
  worker joined. `cargo fmt --all --check`, `clippy --workspace --all-targets -D warnings`
  and `cargo test --workspace` green (49 passed, 1 ignored).
- Decisions: kept egui **glow** (the pinned eframe feature per ADR-0006) and recorded
  that the acceptance wording says "egui/wgpu" — wgpu is a P5 decision, so this task
  satisfies "via egui" only. Decode stays off the UI thread (role rule). The UI drains
  the newest frame and counts skipped frames as dropped, so the status bar cannot
  disagree with the worker's view of loss.
- Next: T-0106 (per-stage latency spans + report) is now the last P1 task; T-0105 and
  T-0107 still need an independent reviewer. Later integration: swap `FakeCapturer` for
  `DxgiCapturer` inside `tools/loopback` (follow-up task; that path was T-0104's).
- Gotchas: the per-frame RGBA→`ColorImage` conversion plus full-texture upload on the
  UI thread is a P1 shortcut — the P5 GPU/NV12 path must replace it. `TextureHandle`
  is recreated when frame dimensions change; a resize mid-stream is handled but not
  tested. Launching the window from a headless/RDP-disconnected session will not render
  (needs a desktop), same constraint as the ignored DXGI test.

## 2026-11-19 04:44 · opencode-a2 · role: qa-bench · T-0106
- Did: made the per-stage latency report real. `RunStats` gained `samples: Vec<StageTimings>`
  (one per presented frame) because sums cannot produce a p50; `tools/vd-bench` now runs
  `vd_loopback::run` end to end and prints nearest-rank p50/p95/p99/max/mean for capture,
  encode, decode, present and total, with frame/drop counters. Replaced the old netsim
  "probe" stub (it printed a hard-coded `lan` latency and measured nothing) and dropped
  its `vd-netsim` dependency. 4 new tests; perf doc has a reproduction block plus the
  three measured rows.
- State: **measured this machine, `--release`, software H.264** — 720p30: total p50
  24.75 ms / p95 27.83 ms, 0 dropped of 300; 720p60: p50 25.31 ms / p95 30.18 ms,
  8 dropped; 1080p30: p50 55.61 ms / p95 61.47 ms, 26 dropped. All drops are
  capture→encode (latest-frame-wins), never downstream. fmt / clippy `-D warnings` /
  `cargo test --workspace` green (53 passed, 1 ignored).
- Decisions: amended T-0106's `Touches` to include `tools/loopback/**` and said so on the
  task, because percentiles require the per-frame samples that only the loopback records;
  T-0104 was already DONE with its lock released, so nothing was taken from an active
  agent. Percentiles are nearest-rank on purpose: a budget should be compared to a value
  that actually occurred, not an interpolated one.
- Next: P1 implementation is now complete (5 DONE + T-0105/T-0106/T-0107 in REVIEW).
  Needs three independent reviews; then P2 (T-0202 quinn) is the next claimable task.
- Gotchas: `present` p50 is 0.00 ms by construction (dimension check until the viewer's
  texture path is measured here) — do not read it as a free render. Capture numbers come
  from `FakeCapturer` (pattern generation), so they are not comparable to the DXGI rows.
  Percentiles exclude dropped frames entirely: they describe presented frames only.
  Software encode cannot hold 720p60 or 1080p30 on this CPU; that is the P5 HW-encode
  trigger, not a queue-policy bug.

## 2026-11-19 04:56 · opencode-a2 · role: network-transport · T-0202
- Did: added `vd-net::quic` — a real QUIC endpoint wrapper. `QuicServer::bind_with` /
  `accept`, `QuicClient::connect` with a pinned server certificate, raw `open_bi` /
  `accept_bi` streams for T-0203/T-0204, ALPN `vd/1`, TLS 1.3 via quinn + rustls, and
  `examples/quic_ping.rs` for a two-process check. New deps (`quinn` 0.11, `rustls`
  0.23 ring-only, `rcgen` 0.13) are crate-local; Tokio is a dev-dependency only because
  the library must not own a runtime.
- State: 5 integration tests green over real UDP loopback (round-trip, unpinned cert
  rejected, unknown server name rejected, zero idle timeout rejected, idle close at the
  configured timeout). Manual two-process run: server + client processes on this machine
  exchanged `ping from vd-net` / `pong from vd-net`. fmt / clippy `-D warnings` /
  `cargo test --workspace` green (58 passed, 1 ignored).
- Decisions: certificate **pinning** instead of a skip-verification verifier — a dev
  self-signed cert is exchanged out of band and pinned in a rustls root store, so the
  insecure option never exists in the API. Idle timeout is applied through
  `TransportConfig::max_idle_timeout` on both ends. `accept_bi` exposes quinn's actual
  signature (no `Option` wrapper); server-side `remote_address()` is the client's
  ephemeral port, which the tests now assert correctly.
- **Knobs (acceptance asks for these in HANDOFF):**
  - `QuicConfig { idle_timeout }` — default `DEFAULT_IDLE_TIMEOUT` = 10 s; zero is
    rejected with `InvalidArgument`; both server and client must set it (each side
    enforces its own).
  - Bind/connect address: `QuicServer::bind(addr)` resolves port 0 to a real port
    (`local_addr()`); `QuicClient::connect(addr, server_name, cert_der, cfg)` picks a
    wildcard local port for the address family.
  - `server_name` must be covered by the certificate SANs: the dev identity covers
    `localhost` and `vd-local`.
  - ALPN is a constant (`quic::ALPN`, `vd/1`); mismatched ALPN fails the handshake.
  - Certificate: `server.cert_der()` → bytes the client passes to `connect` (the example
    writes them to `target/quic-dev-cert.der`).
- Next: T-0203 (Control/Input reliable streams) and T-0204 (Video/Feedback datagrams)
  are now unblocked. P1 still needs three independent reviews (T-0105/T-0106/T-0107);
  T-0202 needs its own reviewer.
- Gotchas: handshake failures only surface when the server side is actually polling
  `accept()` — a test that never accepts waits for the idle timeout instead of a TLS
  error. Dropping `QuicClient` drops its endpoint; use `QuicClient::close` for a clean
  application close. The dev certificate is per-process and unauthenticated: do not
  mistake it for the P3 identity story.

## 2026-11-19 05:54 · opencode-a2 · role: network-transport · T-0205
- Did: added `vd_net::video::fragment_frame` — splits one encoded frame into
  `vd_proto::VideoFrameFragment`s in send order with a 1150 B default cap
  (`DEFAULT_MAX_FRAGMENT_DATA`; `MTU` = 1200), repeated metadata, exact `frag_total`, and
  `InvalidArgument` for empty payloads, zero/oversized dimensions, empty codec ids, a zero
  cap, or frames needing >`u16::MAX` fragments. Tests: 7 module tests + 2 integration
  tests that fragment a **real** `vd_codec::EncodedFrame`.
- State: fmt / clippy `-D warnings` / `cargo test --workspace` green (67 passed, 1
  ignored = the live DXGI capture test).
- Deviations (deliberate):
  - The public input is `EncodedFrameRef`, a borrowed field-for-field view of
    `vd_codec::EncodedFrame`, not the codec type. Architecture §3 allows only downward
    deps, so `vd-net` cannot link `vd-codec` in its product graph without an
    architect/ADR decision. `vd-codec` is a **dev-dependency** only, used to prove the
    acceptance case against the real encoder output. If the architect prefers a direct
    `vd-net → vd-codec` edge, this becomes a one-line signature change.
  - Fragments carry `dirty_rects: None` because `vd_codec::EncodedFrame` has no dirty-rect
    field; capture rects are dropped at the encoder boundary today. Needs a codec/proto
    change (architect), not a fragmenter fix.
- Next: T-0206 reassembler can build on `VideoFrameFragment` directly and share the
  `frame_id`/`frag_total` assumptions; T-0207 pacer consumes the fragment list. Open
  reviews: T-0105/T-0106/T-0107 (P1), T-0202 and T-0205 (P2).
- Gotchas: `frag_idx`/`frag_total` are `u16`, so a frame over ~75 MB at the default cap is
  rejected rather than silently truncated. Each fragment owns a copied `Vec<u8>` — fine
  for now, but a per-frame buffer pool belongs with the P5 buffer work. 1150 B is the data
  budget; keep serialized envelope overhead inside the remaining ~50 B of the 1200 B MTU.

## 2026-11-20 06:15 · cursor-grok · role: network-transport · T-0203
- Did: `vd_net::streams::ReliableChannels` — viewer `open`s two bidirectional QUIC
  streams, host `accept`s them. First byte is `Channel as u8` (Control 0, Input 1) so
  accept order does not matter. After that, each direction is `u32be` length + opaque
  bytes, capped at 64 KiB. Quinn `set_priority` on each side's send half: Input 2,
  Control 1, both above 0 (reserved for later clipboard/file/chat streams). Unknown
  and duplicate tags are reset with application code 1. 5 loopback integration tests.
- State: Control and Input round-trip on separate streams, including an empty Control
  payload, and both ends report Input priority above Control. Oversized sends,
  Video/Feedback, a bad tag, a duplicate Control tag, and a declared length above the
  cap all fail cleanly. `cargo test -p vd-net --all-targets` green (the new tests plus
  the existing quic/video/sim suites). `cargo clippy -p vd-net --all-targets -D warnings`
  clean. Task is REVIEW; `crates/vd-net/**` lock released. Did not mark T-0202 DONE.
- Decisions: payloads stay opaque. Postcard `Envelope` encoding belongs to the app
  (or a later task); `vd-proto` was not edited. The viewer opens and the host accepts,
  which matches "client initiates streams". Priority is per send direction — quinn
  does not copy it to the peer, so `accept` sets it again on the host's send half.
- **Knobs:**
  - `CONTROL_STREAM_PRIORITY = 1`, `INPUT_STREAM_PRIORITY = 2`. Higher is scheduled
    first. Leave `0` for bulk reliable streams.
  - `MAX_RELIABLE_PAYLOAD = 64 KiB`. Larger local sends are `InvalidArgument`; a peer
    that declares a longer frame gets `Protocol` and nothing is allocated for it.
  - Stream tag is the `Channel` discriminant. Only 0 and 1 are accepted here.
  - Bad-tag reset code is `1` (`BAD_STREAM_TAG`).
- Next: T-0204 (Video/Feedback/InputMove/Audio datagrams) on `crates/vd-net/**`, then
  T-0206 reassembler on the T-0205 fragment list. T-0302 is the checked-out branch
  this session started from, but `apps/vd-signal/**` is still locked for T-0301 until
  2026-11-20 10:42 UTC and T-0301 is REVIEW, so that path was left alone. This branch
  `agent/network-transport/T-0203` was cut from that tip because `main` is still the
  init commit and does not contain the QUIC wrapper.
- Gotchas: `accept` waits until both tags arrive; a peer that opens only one stream
  stalls it (the caller can timeout). Dropping `QuicServer` drops the endpoint, so a
  test that still needs to read must keep the server task alive until the read
  finishes. `recv` allocates one `Vec` per message — fine for input, not a pattern to
  copy onto the video path. Stream priority does not keep Input ahead of video;
  video has to stay on datagrams (T-0204) for that.
  `cargo clippy --workspace --all-targets -D warnings` is already red on this tip
  because of `vd-input` (T-0209/T-0210): `MoveCoalescer::feed` is public but
  `DesktopBounds` and `HostInput` are `pub(crate)`, plus dead-code and
  `must_use_unit`. That crate is outside this task's `Touches`. `vd-net` itself is
  clean.


