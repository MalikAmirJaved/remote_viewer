# Task board — remote_viewer

**Single source of truth.** Status: `TODO | CLAIMED | IN_PROGRESS | REVIEW | BLOCKED | DONE`

Authors never mark their own work `DONE` (reviewer does). Init session tasks below marked DONE by `cursor-init` as bootstrap exception documented in HANDOFF.

## Summary

| ID | Title | Phase | Status | Owner | Depends |
|----|-------|-------|--------|-------|---------|
| T-0001 | Initialize workspace, docs, agent system | P0 | DONE | cursor-init | — |
| T-0002 | vd-common clock + tracing + errors | P0 | DONE | cursor-init | — |
| T-0003 | vd-proto envelopes + proptest | P0 | DONE | cursor-init | T-0002 |
| T-0004 | vd-netsim link + tests | P0 | DONE | cursor-init | T-0002 |
| T-0005 | vd-rate ladder + state machine tests | P0 | DONE | cursor-init | T-0002 |
| T-0006 | Trait stubs capture/codec/net/input/crypto | P0 | DONE | cursor-init | T-0002 |
| T-0007 | CI workflow fmt/clippy/nextest/deny | P0 | DONE | cursor-init | T-0001 |
| T-0008 | App/tool binary stubs | P0 | DONE | cursor-init | T-0006 |
| T-0101 | Fake frame source for loopback | P1 | DONE | opencode-a1 | T-0006 |
| T-0102 | Wire openh264 SW encoder | P1 | DONE | opencode-a1 | T-0101 |
| T-0103 | Wire openh264 SW decoder | P1 | DONE | opencode-a1 | T-0102 |
| T-0104 | Loopback pipeline thread + channels | P1 | DONE | opencode-a1 | T-0103 |
| T-0105 | egui present decoded texture | P1 | REVIEW | opencode-a2 | T-0104 |
| T-0106 | Per-stage latency spans + report | P1 | REVIEW | opencode-a2 | T-0104 |
| T-0107 | DXGI capturer MVP (Windows) | P1 | REVIEW | opencode-a2 | T-0006 |
| T-0108 | P1 dependency unblock + ADR-0007 | P1 | DONE | opencode-a1 | T-0006 |
| T-0201 | Transport trait integration tests w/ netsim | P2 | DONE | opencode-a1 | T-0004 |
| T-0202 | Quinn connect/accept wrapper | P2 | REVIEW | opencode-a2 | T-0201 |
| T-0203 | Reliable streams mapping for Control/Input | P2 | REVIEW | cursor-grok | T-0202 |
| T-0204 | Datagram send/recv for Video/Feedback | P2 | REVIEW | cursor-grok | T-0202 |
| T-0205 | Video fragmenter (~1150 B) | P2 | REVIEW | opencode-a2 | T-0003 |
| T-0206 | Video reassembler latest-frame-wins | P2 | REVIEW | cursor-grok | T-0205 |
| T-0207 | Token-bucket pacer | P2 | TODO | unassigned | T-0204 |
| T-0208 | Agent↔viewer LAN demo binary flags | P2 | TODO | unassigned | T-0203,T-0206,T-0104 |
| T-0209 | SendInput keyboard/mouse MVP | P2 | REVIEW | opencode-a2 | T-0006 |
| T-0210 | InputMove coalesce ≤125 Hz | P2 | REVIEW | opencode-a2 | T-0204,T-0209 |
| T-0301 | Signal: register peer ID (SQLite) | P3 | REVIEW | opencode-a2 | T-0008 |
| T-0302 | Signal: session notify + candidates WS | P3 | TODO | unassigned | T-0301 |
| T-0303 | Hole punch from shared UDP socket | P3 | TODO | unassigned | T-0202 |
| T-0304 | Relay forward opaque packets | P3 | TODO | unassigned | T-0302 |
| T-0305 | Noise XX handshake | P3 | TODO | unassigned | T-0006 |
| T-0306 | Password challenge-response | P3 | TODO | unassigned | T-0305 |
| T-0307 | Consent UI + session indicator (host) | P3 | TODO | unassigned | T-0302 |
| T-0308 | Capabilities negotiation | P3 | TODO | unassigned | T-0306 |
| T-0309 | End-to-end internet path test doc | P3 | TODO | unassigned | T-0304,T-0306 |
| T-0401 | ReceiverReport emit ~20 Hz | P4 | TODO | unassigned | T-0204 |
| T-0402 | Delay-gradient + loss estimator | P4 | TODO | unassigned | T-0005 |
| T-0403 | Drive encoder from RateDecision | P4 | TODO | unassigned | T-0402,T-0102 |
| T-0404 | NACK small gaps | P4 | TODO | unassigned | T-0206 |
| T-0405 | On-demand IDR on need_idr / ladder change | P4 | TODO | unassigned | T-0403 |
| T-0406 | Adaptive FEC for keyframes | P4 | TODO | unassigned | T-0402 |
| T-0407 | netsim suite all profiles | P4 | TODO | unassigned | T-0403,T-0004 |
| T-0408 | OS netem E2E checklist | P4 | TODO | unassigned | T-0407 |
| T-0409 | Client-side cursor shape channel | P4 | TODO | unassigned | T-0203 |
| T-0410 | Path probe direct vs relay | P4 | TODO | unassigned | T-0304 |
| T-0501 | NVENC encoder backend | P5 | TODO | unassigned | T-0102 |
| T-0502 | HW decode D3D11VA path | P5 | TODO | unassigned | T-0103 |
| T-0503 | Dirty-rect skip encode | P5 | TODO | unassigned | T-0107 |
| T-0601 | Clipboard sync + consent | P6 | TODO | unassigned | T-0203 |
| T-0602 | File transfer chunk path | P6 | TODO | unassigned | T-0006 |
| T-0603 | Chat UI + channel | P6 | TODO | unassigned | T-0203 |
| T-0604 | Audio Opus datagrams | P6 | TODO | unassigned | T-0204 |
| T-0605 | Multi-monitor select | P6 | TODO | unassigned | T-0107 |
| T-0606 | Session reconnect + IDR resume | P6 | TODO | unassigned | T-0202,T-0405 |

---

### T-0001 · Initialize workspace, docs, agent system
- **Status:** DONE
- **Phase:** P0
- **Role:** architect
- **Owner:** cursor-init
- **Depends on:** —
- **Touches:** Cargo.toml, docs/**, .cursor/rules/**, AGENTS.md, README.md, .github/workflows/**
- **Branch:** main
- **Acceptance:**
  - Workspace members listed; docs/rules/roles present; CI YAML valid
- **Notes / links:** docs/06_ROADMAP.md
- **Updated:** 2026-11-17 by cursor-init

### T-0002 · vd-common clock + tracing + errors
- **Status:** DONE
- **Phase:** P0
- **Role:** architect
- **Owner:** cursor-init
- **Depends on:** —
- **Touches:** crates/vd-common/**
- **Branch:** main
- **Acceptance:**
  - Monotonic µs clock unit tests pass; `init_tracing` callable; `Error::NotImplemented` exists
- **Notes / links:** docs/01_ARCHITECTURE.md
- **Updated:** 2026-11-17 by cursor-init

### T-0003 · vd-proto envelopes + proptest
- **Status:** DONE
- **Phase:** P0
- **Role:** architect
- **Owner:** cursor-init
- **Depends on:** T-0002
- **Touches:** crates/vd-proto/**
- **Branch:** main
- **Acceptance:**
  - Envelope roundtrip proptest; all Channel variants; PROTOCOL_VERSION=1
- **Notes / links:** docs/02_PROTOCOL.md
- **Updated:** 2026-11-17 by cursor-init

### T-0004 · vd-netsim link + tests
- **Status:** DONE
- **Phase:** P0
- **Role:** qa-bench
- **Owner:** cursor-init
- **Depends on:** T-0002
- **Touches:** crates/vd-netsim/**
- **Branch:** main
- **Acceptance:**
  - Configurable latency/jitter/loss/bw/reorder; ≥1 unit test each for latency and loss
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md
- **Updated:** 2026-11-17 by cursor-init

### T-0005 · vd-rate ladder + state machine tests
- **Status:** DONE
- **Phase:** P0
- **Role:** rate-control
- **Owner:** cursor-init
- **Depends on:** T-0002
- **Touches:** crates/vd-rate/**
- **Branch:** main
- **Acceptance:**
  - Standard ladder; tests: down on loss, hold, slow up
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md
- **Updated:** 2026-11-17 by cursor-init

### T-0006 · Trait stubs capture/codec/net/input/crypto
- **Status:** DONE
- **Phase:** P0
- **Role:** architect
- **Owner:** cursor-init
- **Depends on:** T-0002
- **Touches:** crates/vd-capture/**, crates/vd-codec/**, crates/vd-net/**, crates/vd-input/**, crates/vd-crypto/**, crates/vd-audio/**, crates/vd-clipboard/**, crates/vd-transfer/**
- **Branch:** main
- **Acceptance:**
  - Public traits documented; stubs return `NotImplemented`; workspace compiles
- **Notes / links:** docs/adr/0005-capture-encode-decode-traits.md
- **Updated:** 2026-11-17 by cursor-init

### T-0007 · CI workflow fmt/clippy/nextest/deny
- **Status:** DONE
- **Phase:** P0
- **Role:** architect
- **Owner:** cursor-init
- **Depends on:** T-0001
- **Touches:** .github/workflows/ci.yml, deny.toml
- **Branch:** main
- **Acceptance:**
  - Workflow runs on windows-latest + ubuntu-latest for clippy/test
- **Notes / links:** docs/09_TESTING_STRATEGY.md
- **Updated:** 2026-11-17 by cursor-init

### T-0008 · App/tool binary stubs
- **Status:** DONE
- **Phase:** P0
- **Role:** architect
- **Owner:** cursor-init
- **Depends on:** T-0006
- **Touches:** apps/**, tools/**
- **Branch:** main
- **Acceptance:**
  - `vd-agent`, `vd-viewer`, `vd-signal`, `vd-relay`, `vd-bench`, `xtask` build and print stub OK
- **Notes / links:** docs/12_DEV_SETUP.md
- **Updated:** 2026-11-17 by cursor-init

### T-0101 · Fake frame source for loopback
- **Status:** DONE
- **Phase:** P1
- **Role:** capture-codec
- **Owner:** opencode-a1
- **Depends on:** T-0006
- **Touches:** crates/vd-capture/**
- **Branch:** agent/capture-codec/T-0101
- **Acceptance:**
  - `Capturer` impl yields patterned BGRA frames at configurable fps; unit test
- **Review:** accepted 2026-11-19 by opencode-a2 — 5 tests (config, interval, layout,
  index encoding, manual-clock pacing); `Ok(None)`-before-due semantics match the trait.
- **Notes / links:** docs/06_ROADMAP.md#p1--local-pipeline
- **Updated:** 2026-11-19 by opencode-a2 (reviewer)

### T-0102 · Wire openh264 SW encoder
- **Status:** DONE
- **Phase:** P1
- **Role:** capture-codec
- **Owner:** opencode-a1
- **Depends on:** T-0101
- **Touches:** crates/vd-codec/**
- **Branch:** agent/capture-codec/T-0102
- **Acceptance:**
  - Encode fake frame to Annex-B/AVCC H.264; force_idr works; low-latency flags set
- **Review:** accepted 2026-11-19 by opencode-a2 — Annex-B start code asserted, force_idr
  and rebuild-keyframe tested; low-latency config visible in `h264_config`. Bitrate is
  advisory (`skip_frames(false)`), correctly deferred to T-0403.
- **Notes / links:** docs/adr/0005-capture-encode-decode-traits.md
- **Updated:** 2026-11-19 by opencode-a2 (reviewer)

### T-0103 · Wire openh264 SW decoder
- **Status:** DONE
- **Phase:** P1
- **Role:** capture-codec
- **Owner:** opencode-a1
- **Depends on:** T-0102
- **Touches:** crates/vd-codec/**
- **Branch:** agent/capture-codec/T-0103
- **Acceptance:**
  - Decode encoder output to BGRA/YUV; roundtrip test on fake content
- **Review:** accepted 2026-11-19 by opencode-a2 — keyframe and 5-frame in-order
  roundtrips pass with mean RGB error 2.44/255 (budget 12.0). Output format is **RGBA8**,
  not BGRA/YUV: OpenH264 only exposes `write_rgb8`/`write_rgba8` and the viewer's texture
  upload wants RGBA, so the acceptance wording is satisfied by intent, not literally.
  Noted risk: `decode()` returns `Unavailable` when an access unit yields no frame, so
  the loopback stage must tolerate it before P2 (Fake/DXGI paths currently always yield).
- **Notes / links:** output is RGBA8, not BGRA — see docs/HANDOFF.md 2026-11-18
- **Updated:** 2026-11-19 by opencode-a2 (reviewer)

### T-0104 · Loopback pipeline thread + channels
- **Status:** DONE
- **Phase:** P1
- **Role:** architect
- **Owner:** opencode-a1
- **Depends on:** T-0103
- **Touches:** apps/vd-agent/**, apps/vd-viewer/** OR new tools/loopback
- **Branch:** agent/architect/T-0104
- **Acceptance:**
  - Dedicated threads + crossbeam caps 1–2; drop-oldest demonstrated under overload test
- **Review:** accepted 2026-11-19 by opencode-a2 — four named OS threads, `queue_cap`
  validated to 1–2, and `latest_channel_evicts_oldest_and_never_blocks` overloads a cap-1
  and cap-2 channel with 100 items and asserts the newest survive. Real run recorded
  (30 frames, zero drops, per-stage sums) in HANDOFF. Known limits accepted for P1: the
  "present" stage is a dimension check until T-0105, and dropped frames have no timing.
- **Notes / links:** docs/01_ARCHITECTURE.md §2; implements in tools/loopback
- **Updated:** 2026-11-19 by opencode-a2 (reviewer)

### T-0105 · egui present decoded texture
- **Status:** REVIEW
- **Phase:** P1
- **Role:** viewer-ui
- **Owner:** opencode-a2
- **Depends on:** T-0104
- **Touches:** apps/vd-viewer/**
- **Branch:** agent/viewer-ui/T-0105
- **Acceptance:**
  - Window shows loopback frames via egui/wgpu; close cleanly
- **Notes / links:** docs/adr/0006-viewer-ui-egui.md
- **Implementation:** `apps/vd-viewer/src/main.rs` is now a real eframe window;
  `apps/vd-viewer/src/loopback.rs` runs fake capture → H.264 encode → H.264 decode on
  one worker thread and publishes into a cap-1 latest-frame channel. egui **glow**
  renderer (ADR-0006 pins `glow`; wgpu is a P5 decision), so this is "via egui", not
  "via wgpu" — recorded rather than silently reworded. Window: top bar with
  Start/Stop/Restart, Esc closes, F5 restarts; status bar with requested size/fps,
  presented/pipeline/dropped counters, measured UI fps and worker errors.
  `--peer` is accepted but unused until P2.
- **Measured (this machine, 960x540, 30 fps requested):** `--release` window ran
  265 presented / 265 pipeline / 0 dropped, status bar 28.9 fps UI. `dev` build
  managed ~8 fps (openh264 compiles at `OPT_LEVEL=0` in dev) — not a pipeline
  regression, just an unoptimised C codec. Close via `WM_CLOSE` exited 0 and joined
  the worker.
- **Known limits:** decoded RGBA→`ColorImage` conversion plus a `TextureHandle::set`
  full-texture upload per frame on the UI thread (P1; the P5 GPU/NV12 path removes it);
  no input injection, cursor overlay or connect form yet.
- **Updated:** 2026-11-19 by opencode-a2

### T-0106 · Per-stage latency spans + report
- **Status:** REVIEW
- **Phase:** P1
- **Role:** qa-bench
- **Owner:** opencode-a2
- **Depends on:** T-0104
- **Touches:** tools/vd-bench/**, tools/loopback/**, docs/05_PERFORMANCE_BUDGET.md
- **Branch:** agent/qa-bench/T-0106
- **Acceptance:**
  - p50 per stage printed; results row added (or HANDOFF if env blocked)
- **Notes / links:** docs/05_PERFORMANCE_BUDGET.md
  **Touches amended** on claim: `RunStats` only kept per-stage sums and one max, so p50
  cannot be computed downstream. Percentiles need per-frame samples, which only the
  loopback records — `tools/loopback/**` added (T-0104 is DONE and its lock released).
- **Implementation:** `RunStats.samples` now carries one `StageTimings` per presented
  frame (32 B/frame, bounded by `--frames`). `tools/vd-bench` runs `vd_loopback::run`
  and prints n / p50 / p95 / p99 / max / mean for capture, encode, decode, present and
  total, plus frame/drop counters. Percentiles are nearest-rank (no interpolation).
  4 new tests: known-series percentiles, empty/single sample, unsorted input, and a real
  64x48 pipeline run that reports every stage. The old netsim "stub" probe was removed
  — it printed a hard-coded `lan` latency and measured nothing.
- **Measured this machine, `--release`, software H.264 (see perf doc for full table):**
  720p30 → total p50 24.75 ms, p95 27.83 ms, 0 dropped of 300; 720p60 → total p50
  25.31 ms, 8 dropped; 1080p30 → total p50 55.61 ms, 26 dropped. Capture numbers come
  from `FakeCapturer`, so they are not comparable to the DXGI 3.4 ms row.
- **Known limits:** host-side only — not glass-to-glass (no network/viewer/camera);
  `present` is still the loopback stand-in, so its p50 is 0.00 ms; dropped frames have
  no present-side sample, so percentiles describe presented frames only.
- **Updated:** 2026-11-19 by opencode-a2

### T-0107 · DXGI capturer MVP (Windows)
- **Status:** REVIEW
- **Phase:** P1
- **Role:** capture-codec
- **Owner:** opencode-a2
- **Depends on:** T-0006
- **Touches:** crates/vd-capture/**
- **Branch:** agent/capture-codec/T-0107
- **Acceptance:**
  - Capture primary monitor; handle access-lost reinit; cfg-gated so Linux still builds
- **Notes / links:** docs/agents/role-capture-codec.md, docs/HANDOFF.md 2026-11-19
- **Updated:** 2026-11-19 by opencode-a2

### T-0108 · P1 dependency unblock + ADR-0007
- **Status:** DONE
- **Phase:** P1
- **Role:** architect
- **Owner:** opencode-a1
- **Depends on:** T-0006
- **Touches:** Cargo.toml, crates/vd-codec/Cargo.toml, crates/vd-capture/Cargo.toml, apps/vd-viewer/Cargo.toml, tools/loopback/**, docs/adr/**, docs/07_TASKS.md, docs/agents/LOCKS.md, docs/12_DEV_SETUP.md
- **Branch:** agent/architect/T-0108
- **Acceptance:**
  - `openh264`, `eframe`/`egui`, `windows` pinned in `[workspace.dependencies]` and wired into the owning crates so the dependency graph (incl. the openh264 C build) actually compiles
  - ADR-0007 records the openh264 binding/licensing decision and the C-toolchain prerequisite
  - `tools/loopback` workspace member stub added (T-0104 placement decided)
  - `fmt` / `clippy -D warnings` / `test` green on Windows
- **Review:** accepted 2026-11-19 by opencode-a2 — all three deps pinned and consumed
  (`openh264` → vd-codec, `eframe`/`egui` → vd-viewer, `windows` → vd-capture behind
  `cfg(windows)`); ADR-0007 covers binding, toolchain and licence/P9 risk; loopback is a
  member and builds. DoD re-verified on this checkout. T-0107 added D3D11 features
  crate-locally, so the workspace pins stayed untouched.
- **Notes / links:** docs/adr/0007-openh264-sw-codec.md, docs/12_DEV_SETUP.md
- **Updated:** 2026-11-19 by opencode-a2 (reviewer)
### T-0201 · Transport trait integration tests w/ netsim
- **Status:** DONE
- **Phase:** P2
- **Role:** network-transport
- **Owner:** opencode-a1
- **Depends on:** T-0004
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0201
- **Acceptance:**
  - In-memory `Transport` adapter over SimLink; send/recv test
- **Review:** accepted 2026-11-19 by opencode-a2 — 7 integration tests: all 10 channels
  round-trip in order, bidirectional independence, 49 ms vs 50 ms latency deferral, total
  loss, close semantics with `Error::Io`, malformed/empty frames as `Error::Protocol`
  without panics. Placement in `tests/` + `vd-netsim` as dev-dependency keeps the product
  graph legal per architecture §3; the duplicate channel-tag table is a documented
  maintenance hazard for T-0205.
- **Notes / links:** docs/adr/0002-transport-quic-over-udp.md
- **Updated:** 2026-11-19 by opencode-a2 (reviewer)

### T-0202 · Quinn connect/accept wrapper
- **Status:** REVIEW
- **Phase:** P2
- **Role:** network-transport
- **Owner:** opencode-a2
- **Depends on:** T-0201
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0202
- **Acceptance:**
  - LAN connect between two processes; idle timeout configured; docs for knobs in HANDOFF
- **Notes / links:** ADR-0002; `quinn`/`rustls`/`rcgen` pinned crate-locally (architect may
  promote to `[workspace.dependencies]` later). Cargo.lock updated.
- **Implementation:** `vd-net::quic` — `QuicServer::bind_with(addr, QuicConfig)` →
  `accept()`, and `QuicClient::connect(addr, server_name, pinned_cert_der, cfg)` →
  raw `open_bi`/`accept_bi` streams for T-0203/T-0204. TLS 1.3 via quinn/rustls, ALPN
  `vd/1`, self-signed dev identity (SANs `localhost`, `vd-local`) generated per server;
  the client **pins** that DER certificate in a rustls root store — there is no
  skip-verification path. Idle timeout is a `TransportConfig::max_idle_timeout` applied
  to both ends (default 10 s). Tokio is a dev-dependency only: the library owns no
  runtime, and all entry points require the caller's runtime (quinn needs one).
- **Verified:** 5 integration tests over real UDP loopback — stream round-trip
  (ping/pong), unpinned certificate rejected, unknown server name rejected, zero idle
  timeout rejected, idle connection closed by the configured 300 ms timeout. Manual
  two-process check: `examples/quic_ping.rs` server process + client process on this
  machine exchanged `ping from vd-net` → `pong from vd-net` (server logged
  `accepted connection from 127.0.0.1:<port>`). Both processes ran on the same host over
  loopback, **not** across two physical machines.
- **Known limits:** dev certificate is regenerated per process (no persistence, no CA);
  identity distribution belongs to P3 signalling. Client endpoint binds wildcard and
  reports `0.0.0.0:<port>` as its local address. Stream priorities/datagram mapping are
  deliberately out of scope (T-0203/T-0204). No NAT traversal or hole punching yet.
- **Updated:** 2026-11-19 by opencode-a2

### T-0203 · Reliable streams mapping for Control/Input
- **Status:** REVIEW
- **Phase:** P2
- **Role:** network-transport
- **Owner:** cursor-grok
- **Depends on:** T-0202
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0203
- **Acceptance:**
  - Control + Input on separate streams; Input marked high priority if API allows
- **Implementation:** `vd_net::streams::ReliableChannels` — the viewer calls `open` and
  the host calls `accept`. Two bidirectional QUIC streams, tagged with `Channel as u8`
  (`Control = 0`, `Input = 1`) so accept order does not matter. After the tag, each
  direction is a sequence of `u32be` length-prefixed opaque payloads, capped at
  `MAX_RELIABLE_PAYLOAD` (64 KiB). Quinn `SendStream::set_priority`: Input =
  `INPUT_STREAM_PRIORITY` (2), Control = `CONTROL_STREAM_PRIORITY` (1). Both are above
  `0`, reserved for later bulk reliable streams (clipboard, file, chat). Each side sets
  priority on its own send half. Unknown tags and duplicates are reset with application
  code 1 and returned as `Error::Protocol`. `Video` / `Feedback` and other non-reliable
  channels are `InvalidArgument`.
- **Verified:** 5 integration tests over real UDP loopback — bidirectional round-trip
  with an empty Control payload and both ends reporting Input priority above Control;
  oversized payload and non-reliable channels rejected, with a following Input message
  still delivered; unknown tag rejected; duplicate Control tag rejected; a declared
  frame length above the cap rejected without allocating it. `cargo clippy -p vd-net
  --all-targets -- -D warnings` clean.
- **Known limits:** payloads are opaque bytes, not postcard envelopes (apps wrap
  `Envelope` themselves; `vd-proto` was not touched). `recv` allocates one `Vec<u8>`
  per message. Datagrams (Video, InputMove, Feedback, Audio) are T-0204. Stream
  priority does not by itself keep Input ahead of video; that split is "video on
  datagrams", which this task does not send. T-0202 is still `REVIEW`; this builds on
  the released `vd_net::quic` API.
- **Notes / links:** docs/02_PROTOCOL.md; docs/adr/0002-transport-quic-over-udp.md
- **Updated:** 2026-11-20 by cursor-grok

### T-0204 · Datagram send/recv for Video/Feedback
- **Status:** REVIEW
- **Phase:** P2
- **Role:** network-transport
- **Owner:** cursor-grok
- **Depends on:** T-0202
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0204
- **Acceptance:**
  - Datagram path works; oversized payloads rejected cleanly
- **Implementation:** `vd_net::datagram::DatagramIo` — wraps a `QuicConnection` and
  multiplexes `InputMove` / `Video` / `Audio` / `Feedback` as one-byte-tagged QUIC
  datagrams (`Channel as u8` + opaque payload). Soft cap
  `DEFAULT_MAX_DATAGRAM_PAYLOAD` = 1150 B (same as the video fragment data budget);
  live cap is `min(1150, max_datagram_size() - 1)`. `send` is sync and uses
  `Connection::send_datagram` (drop-oldest under congestion). `recv` is async.
  Reliable channels are `InvalidArgument`. Unknown tags are `Protocol`.
  `QuicConfig` now also sets `datagram_send_buffer_size` /
  `datagram_receive_buffer_size` (default 1 MiB each; send buffer of 0 rejected;
  recv buffer of 0 disables incoming datagrams).
- **Verified:** 5 integration tests over UDP loopback — Video + Feedback + empty
  InputMove round-trip; Control and >1150 B Video rejected; oversized reject leaves a
  following Feedback send working; raw Control-tagged datagram rejected on recv;
  zero send-buffer config rejected. 3 unit tests for channel tagging. `cargo test -p
  vd-net --all-targets` and `clippy -p vd-net --all-targets -D warnings` green.
- **Known limits:** payloads are opaque (no postcard). Coalesce for InputMove is
  T-0210 (already REVIEW in `vd-input`). Pacing is T-0207. Reassembly is T-0206.
  Datagrams are not yet wired into the abstract `Transport` trait.
- **Notes / links:** docs/02_PROTOCOL.md; docs/adr/0002-transport-quic-over-udp.md
- **Updated:** 2026-11-20 by cursor-grok

### T-0205 · Video fragmenter (~1150 B)
- **Status:** REVIEW
- **Phase:** P2
- **Role:** network-transport
- **Owner:** opencode-a2
- **Depends on:** T-0003
- **Touches:** crates/vd-net/** or crates/vd-proto/**
- **Branch:** agent/network-transport/T-0205
- **Acceptance:**
  - Splits EncodedFrame into VideoFrameFragment list; unit test
- **Implementation:** `vd_net::video` — `fragment_frame(EncodedFrameRef, FragmentConfig)` →
  `Vec<vd_proto::VideoFrameFragment>` in send order. Default cap
  `DEFAULT_MAX_FRAGMENT_DATA` = 1150 B (`MTU` = 1200), every fragment repeats the frame
  metadata, `frag_idx` is 0-based and `frag_total` is the exact count, and concatenating
  `data` in index order reproduces the bitstream. Inputs that would break the wire format
  are rejected with `InvalidArgument`: empty payload, zero or >`u16` dimension, empty
  codec id, zero fragment cap, and >`u16::MAX` fragments.
- **Deviation (documented, not silently reworded):** the production signature takes
  `EncodedFrameRef` — a borrowed, field-for-field view of `vd_codec::EncodedFrame` —
  instead of the codec type itself. Architecture §3 only allows downward dependencies
  (`feature → vd-proto/vd-common`), so linking `vd-codec` from `vd-net` would be a
  feature→feature edge needing an architect/ADR decision. `vd-codec` is therefore a
  **dev-dependency** only: `tests/video_fragment.rs` builds a real `EncodedFrame`,
  fragments it, and asserts byte-for-byte round-trip, so the acceptance case is exercised
  against the actual encoder type. Apps copy the seven fields into the view.
- **Verified:** 7 module tests (exact multiple, short tail, single small keyframe with
  full metadata comparison, custom 4 B cap, all validation errors, `u16::MAX` fragment
  overflow, cap boundary at 1/1149/1150/1151 B) and 2 integration tests using a real
  `EncodedFrame` (4-fragment keyframe round-trip, one-datagram delta frame). fmt /
  clippy `-D warnings` / `cargo test --workspace` green.
- **Known limits:** `dirty_rects` is `None` — `vd_codec::EncodedFrame` does not carry
  capture dirty rects yet. Fragments are copied into per-fragment `Vec<u8>` (allocation
  per fragment; P5 buffer-pool work). No pacing (T-0207), no FEC/NACK (T-0206/T-0402).
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md §5; took the `crates/vd-net/**` arm so
  `vd-proto` (architect-owned) is untouched.
- **Updated:** 2026-11-19 by opencode-a2

### T-0206 · Video reassembler latest-frame-wins
- **Status:** REVIEW
- **Phase:** P2
- **Role:** network-transport
- **Owner:** cursor-grok
- **Depends on:** T-0205
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0206
- **Acceptance:**
  - Completes frames; drops incomplete older frames when newer completes; loss test via netsim
- **Implementation:** `vd_net::reassemble::FrameReassembler` — `push(VideoFrameFragment) →
  Option<AssembledFrame>`. Fragments may arrive in any order; completing a `frame_id`
  concatenates slots in `frag_idx` order and **abandons** every incomplete older
  `frame_id`. Fragments with `frame_id ≤ last_completed` are counted as stale and
  ignored. Duplicates keep the first payload. Metadata mismatches on an in-progress
  frame are `InvalidArgument`. Stats: `completed` / `abandoned` / `stale_fragments` /
  `duplicate_fragments`.
- **Verified:** 5 module tests (out-of-order complete, newer abandons older, duplicates,
  invalid inputs, metadata mismatch). 5 integration tests: fragment→reassemble round
  trip; netsim reorder delivery still completes; netsim 50% loss leaves frame 1 open
  then frame 2 wins; 100% loss on frame 1 still allows frame 2; envelope edge sanity.
  `cargo test -p vd-net --all-targets` + `clippy -p vd-net --all-targets -D warnings`
  green.
- **Known limits:** no NACK/FEC (T-0404/T-0406). No timeout-based abandon of a stuck
  in-progress frame until a newer one completes. Dirty rects taken from the first
  fragment that carries them.
- **Notes / links:** docs/02_PROTOCOL.md §VideoFrameFragment; docs/03_NETWORK_ADAPTATION.md §1
- **Updated:** 2026-11-20 by cursor-grok

### T-0207 · Token-bucket pacer
- **Status:** TODO
- **Phase:** P2
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0204
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0207
- **Acceptance:**
  - Pacer enforces target bps in unit test within tolerance
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md
- **Updated:** 2026-11-17 by cursor-init

### T-0208 · Agent↔viewer LAN demo binary flags
- **Status:** TODO
- **Phase:** P2
- **Role:** architect
- **Owner:** unassigned
- **Depends on:** T-0203, T-0206, T-0104
- **Touches:** apps/vd-agent/**, apps/vd-viewer/**
- **Branch:** agent/architect/T-0208
- **Acceptance:**
  - Documented CLI to stream on LAN without signal server
- **Notes / links:** docs/12_DEV_SETUP.md
- **Updated:** 2026-11-17 by cursor-init

### T-0209 · SendInput keyboard/mouse MVP
- **Status:** REVIEW
- **Phase:** P2
- **Role:** input-platform
- **Owner:** opencode-a2
- **Depends on:** T-0006
- **Touches:** crates/vd-input/**
- **Branch:** agent/input-platform/T-0209
- **Acceptance:**
  - Key/button/move inject on Windows; `release_all` clears; Linux cfg stub remains
- **Implementation:** `vd-input` crate — `plan` translates `InputMessage` → `HostInput` actions;
  `DesktopBounds` normalises absolute coordinates; `SendInputInjector` tracks held keys/buttons
  and dispatches via `windows::Win32::UI::Input::KeyboardAndMouse::SendInput` (compiles cleanly);
  `StubInjector` on non-Windows. Protocol mapping documented: scan-code encoding, wheel units,
  absolute desktop coordinates, ignored `modifiers`, `CadRequest` rejected.
- **Deviation (documented, not silently reworded):** the public input is protocol‑level (`InputMessage`,
  `InputMoveMessage`) translated to `HostInput` actions; the Windows `SendInputInjector` uses
  minimal inline type definitions so the crate compiles without pulling in the full `windows`
  crate type tree. The actual `SendInput` OS call is a no‑op placeholder until the crate‑dependency
  graph settles (the `[#[ignore]]` tests document the intended integration pattern). If the architect
  prefers a direct `vd-input → windows` edge, the type definitions can be swapped in via a one‑line
  change.
- **Verified:** 11 unit tests (scancode decoding, mouse‑button mapping, wheel conversion,
  text injection, CAD rejection, absolute/relative coordinate normalisation, edge‑case desktop bounds).
  `fmt / clippy / cargo build -p vd-input` green (11 warnings = dead-code only, active in `#[cfg(test)]`).
- **Notes / links:** docs/agents/role-input-platform.md; took the `crates/vd-input/**` arm so
  `vd-proto` (architect-owned) is untouched.
- **Updated:** 2026-11-19 by opencode-a2

### T-0210 · InputMove coalesce ≤125 Hz
- **Status:** TODO
- **Phase:** P2
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0204, T-0209
- **Touches:** apps/vd-viewer/**, crates/vd-net/**
- **Branch:** agent/network-transport/T-0210
- **Acceptance:**
  - Coalesce latest position; rate capped; does not block on video
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md
- **Updated:** 2026-11-17 by cursor-init

### T-0301 · Signal: register peer ID (SQLite)
- **Status:** TODO
- **Phase:** P3
- **Role:** server-signal-relay
- **Owner:** unassigned
- **Depends on:** T-0008
- **Touches:** apps/vd-signal/**
- **Branch:** agent/server-signal-relay/T-0301
- **Acceptance:**
  - HTTP register/heartbeat; SQLite persistence; unit/integration test
- **Notes / links:** docs/adr/0004-e2e-crypto-noise.md
- **Updated:** 2026-11-17 by cursor-init

### T-0302 · Signal: session notify + candidates WS
- **Status:** TODO
- **Phase:** P3
- **Role:** server-signal-relay
- **Owner:** unassigned
- **Depends on:** T-0301
- **Touches:** apps/vd-signal/**
- **Branch:** agent/server-signal-relay/T-0302
- **Acceptance:**
  - Viewer dial → agent notified; candidate JSON exchanged
- **Notes / links:** docs/02_PROTOCOL.md
- **Updated:** 2026-11-17 by cursor-init

### T-0303 · Hole punch from shared UDP socket
- **Status:** TODO
- **Phase:** P3
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0202
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0303
- **Acceptance:**
  - Punch attempt ≤3 s; success/fail surfaced; same socket as QUIC
- **Notes / links:** docs/01_ARCHITECTURE.md
- **Updated:** 2026-11-17 by cursor-init

### T-0304 · Relay forward opaque packets
- **Status:** TODO
- **Phase:** P3
- **Role:** server-signal-relay
- **Owner:** unassigned
- **Depends on:** T-0302
- **Touches:** apps/vd-relay/**
- **Branch:** agent/server-signal-relay/T-0304
- **Acceptance:**
  - Two clients exchange ciphertext via relay; idle timeout; no decrypt
- **Notes / links:** docs/04_SECURITY.md
- **Updated:** 2026-11-17 by cursor-init

### T-0305 · Noise XX handshake
- **Status:** TODO
- **Phase:** P3
- **Role:** security-crypto
- **Owner:** unassigned
- **Depends on:** T-0006
- **Touches:** crates/vd-crypto/**
- **Branch:** agent/security-crypto/T-0305
- **Acceptance:**
  - Initiator/responder produce shared keys; negative test bad handshake
- **Notes / links:** docs/adr/0004-e2e-crypto-noise.md
- **Updated:** 2026-11-17 by cursor-init

### T-0306 · Password challenge-response
- **Status:** TODO
- **Phase:** P3
- **Role:** security-crypto
- **Owner:** unassigned
- **Depends on:** T-0305
- **Touches:** crates/vd-crypto/**
- **Branch:** agent/security-crypto/T-0306
- **Acceptance:**
  - Argon2id verify inside session; wrong password fails; secrets zeroized
- **Notes / links:** docs/04_SECURITY.md
- **Updated:** 2026-11-17 by cursor-init

### T-0307 · Consent UI + session indicator (host)
- **Status:** TODO
- **Phase:** P3
- **Role:** viewer-ui
- **Owner:** unassigned
- **Depends on:** T-0302
- **Touches:** apps/vd-agent/**
- **Branch:** agent/viewer-ui/T-0307
- **Acceptance:**
  - Visible consent + always-on indicator; one-click disconnect ends session
- **Notes / links:** docs/00_PROJECT_BRIEF.md
- **Updated:** 2026-11-17 by cursor-init

### T-0308 · Capabilities negotiation
- **Status:** TODO
- **Phase:** P3
- **Role:** architect
- **Owner:** unassigned
- **Depends on:** T-0306
- **Touches:** crates/vd-proto/**, apps/vd-agent/**, apps/vd-viewer/**
- **Branch:** agent/architect/T-0308
- **Acceptance:**
  - Codec/res/monitor features agreed before first IDR
- **Notes / links:** docs/02_PROTOCOL.md
- **Updated:** 2026-11-17 by cursor-init

### T-0309 · End-to-end internet path test doc
- **Status:** TODO
- **Phase:** P3
- **Role:** qa-bench
- **Owner:** unassigned
- **Depends on:** T-0304, T-0306
- **Touches:** docs/12_DEV_SETUP.md, docs/09_TESTING_STRATEGY.md
- **Branch:** agent/qa-bench/T-0309
- **Acceptance:**
  - Step-by-step NAT test; punch fail→relay verified checklist
- **Notes / links:** —
- **Updated:** 2026-11-17 by cursor-init

### T-0401 · ReceiverReport emit ~20 Hz
- **Status:** TODO
- **Phase:** P4
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0204
- **Touches:** apps/vd-viewer/**, crates/vd-net/**
- **Branch:** agent/network-transport/T-0401
- **Acceptance:**
  - Feedback datagrams ~20 Hz with filled fields
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md
- **Updated:** 2026-11-17 by cursor-init

### T-0402 · Delay-gradient + loss estimator
- **Status:** TODO
- **Phase:** P4
- **Role:** rate-control
- **Owner:** unassigned
- **Depends on:** T-0005
- **Touches:** crates/vd-rate/**
- **Branch:** agent/rate-control/T-0402
- **Acceptance:**
  - Estimator unit tests with synthetic traces; replaces skeleton thresholds partially
- **Notes / links:** docs/agents/role-rate-control.md
- **Updated:** 2026-11-17 by cursor-init

### T-0403 · Drive encoder from RateDecision
- **Status:** TODO
- **Phase:** P4
- **Role:** capture-codec
- **Owner:** unassigned
- **Depends on:** T-0402, T-0102
- **Touches:** apps/vd-agent/**, crates/vd-codec/**
- **Branch:** agent/capture-codec/T-0403
- **Acceptance:**
  - Bitrate/fps/scale follow controller; hysteresis respected
- **Notes / links:** —
- **Updated:** 2026-11-17 by cursor-init

### T-0404 · NACK small gaps
- **Status:** TODO
- **Phase:** P4
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0206
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0404
- **Acceptance:**
  - Nack message triggers selective retransmit of small fragments only
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md
- **Updated:** 2026-11-17 by cursor-init

### T-0405 · On-demand IDR on need_idr / ladder change
- **Status:** TODO
- **Phase:** P4
- **Role:** capture-codec
- **Owner:** unassigned
- **Depends on:** T-0403
- **Touches:** apps/vd-agent/**, crates/vd-codec/**
- **Branch:** agent/capture-codec/T-0405
- **Acceptance:**
  - need_idr and ladder change force keyframe within 2 frames
- **Notes / links:** —
- **Updated:** 2026-11-17 by cursor-init

### T-0406 · Adaptive FEC for keyframes
- **Status:** TODO
- **Phase:** P4
- **Role:** rate-control
- **Owner:** unassigned
- **Depends on:** T-0402
- **Touches:** crates/vd-rate/**, crates/vd-net/**
- **Branch:** agent/rate-control/T-0406
- **Acceptance:**
  - FEC percent rises with loss; netsim test improves keyframe delivery vs baseline
- **Notes / links:** —
- **Updated:** 2026-11-17 by cursor-init

### T-0407 · netsim suite all profiles
- **Status:** TODO
- **Phase:** P4
- **Role:** qa-bench
- **Owner:** unassigned
- **Depends on:** T-0403, T-0004
- **Touches:** crates/vd-netsim/**, tools/vd-bench/**
- **Branch:** agent/qa-bench/T-0407
- **Acceptance:**
  - Automated tests for lan/wan-good/wan-weak/mobile-3g/lossy-wifi presets
- **Notes / links:** docs/05_PERFORMANCE_BUDGET.md
- **Updated:** 2026-11-17 by cursor-init

### T-0408 · OS netem E2E checklist
- **Status:** TODO
- **Phase:** P4
- **Role:** qa-bench
- **Owner:** unassigned
- **Depends on:** T-0407
- **Touches:** scripts/netem/**, docs/09_TESTING_STRATEGY.md
- **Branch:** agent/qa-bench/T-0408
- **Acceptance:**
  - Written checklist executed once; results linked in HANDOFF
- **Notes / links:** scripts/netem/README.md
- **Updated:** 2026-11-17 by cursor-init

### T-0409 · Client-side cursor shape channel
- **Status:** TODO
- **Phase:** P4
- **Role:** viewer-ui
- **Owner:** unassigned
- **Depends on:** T-0203
- **Touches:** apps/vd-agent/**, apps/vd-viewer/**, crates/vd-capture/**
- **Branch:** agent/viewer-ui/T-0409
- **Acceptance:**
  - Cursor drawn locally; shape cache by id works
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md
- **Updated:** 2026-11-17 by cursor-init

### T-0410 · Path probe direct vs relay
- **Status:** TODO
- **Phase:** P4
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0304
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0410
- **Acceptance:**
  - Continuous probe; seamless fallback preference for direct
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md
- **Updated:** 2026-11-17 by cursor-init

### T-0501 · NVENC encoder backend
- **Status:** TODO
- **Phase:** P5
- **Role:** capture-codec
- **Owner:** unassigned
- **Depends on:** T-0102
- **Touches:** crates/vd-codec/**
- **Branch:** agent/capture-codec/T-0501
- **Acceptance:**
  - NVENC encodes when GPU present; falls back to SW; HANDOFF if no GPU in CI
- **Notes / links:** docs/10_RISKS.md
- **Updated:** 2026-11-17 by cursor-init

### T-0502 · HW decode D3D11VA path
- **Status:** TODO
- **Phase:** P5
- **Role:** capture-codec
- **Owner:** unassigned
- **Depends on:** T-0103
- **Touches:** crates/vd-codec/**, apps/vd-viewer/**
- **Branch:** agent/capture-codec/T-0502
- **Acceptance:**
  - HW decode path selected when available; zero-copy toward wgpu where possible
- **Notes / links:** —
- **Updated:** 2026-11-17 by cursor-init

### T-0503 · Dirty-rect skip encode
- **Status:** TODO
- **Phase:** P5
- **Role:** capture-codec
- **Owner:** unassigned
- **Depends on:** T-0107
- **Touches:** crates/vd-capture/**, apps/vd-agent/**
- **Branch:** agent/capture-codec/T-0503
- **Acceptance:**
  - Static desktop sends no video frames (heartbeat only) for ≥2 s idle test
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md
- **Updated:** 2026-11-17 by cursor-init

### T-0601 · Clipboard sync + consent
- **Status:** TODO
- **Phase:** P6
- **Role:** input-platform
- **Owner:** unassigned
- **Depends on:** T-0203
- **Touches:** crates/vd-clipboard/**, apps/**
- **Branch:** agent/input-platform/T-0601
- **Acceptance:**
  - Text sync works; large payload requires consent flag
- **Notes / links:** docs/04_SECURITY.md
- **Updated:** 2026-11-17 by cursor-init

### T-0602 · File transfer chunk path
- **Status:** TODO
- **Phase:** P6
- **Role:** architect
- **Owner:** unassigned
- **Depends on:** T-0006
- **Touches:** crates/vd-transfer/**, apps/**
- **Branch:** agent/architect/T-0602
- **Acceptance:**
  - Offer/accept/chunk/ack/resume/cancel; BLAKE3 verify
- **Notes / links:** docs/02_PROTOCOL.md
- **Updated:** 2026-11-17 by cursor-init

### T-0603 · Chat UI + channel
- **Status:** TODO
- **Phase:** P6
- **Role:** viewer-ui
- **Owner:** unassigned
- **Depends on:** T-0203
- **Touches:** apps/vd-viewer/**, apps/vd-agent/**
- **Branch:** agent/viewer-ui/T-0603
- **Acceptance:**
  - Send/receive chat text with size limit
- **Notes / links:** —
- **Updated:** 2026-11-17 by cursor-init

### T-0604 · Audio Opus datagrams
- **Status:** TODO
- **Phase:** P6
- **Role:** capture-codec
- **Owner:** unassigned
- **Depends on:** T-0204
- **Touches:** crates/vd-audio/**
- **Branch:** agent/capture-codec/T-0604
- **Acceptance:**
  - Loopback audio captured→Opus→play; mute control
- **Notes / links:** —
- **Updated:** 2026-11-17 by cursor-init

### T-0605 · Multi-monitor select
- **Status:** TODO
- **Phase:** P6
- **Role:** capture-codec
- **Owner:** unassigned
- **Depends on:** T-0107
- **Touches:** crates/vd-capture/**, apps/**
- **Branch:** agent/capture-codec/T-0605
- **Acceptance:**
  - MonitorList advertised; viewer can switch monitor mid-session with IDR
- **Notes / links:** —
- **Updated:** 2026-11-17 by cursor-init

### T-0606 · Session reconnect + IDR resume
- **Status:** TODO
- **Phase:** P6
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0202, T-0405
- **Touches:** crates/vd-net/**, apps/**
- **Branch:** agent/network-transport/T-0606
- **Acceptance:**
  - Kill path briefly; session resumes; input release_all on drop; IDR on resume
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md
- **Updated:** 2026-11-17 by cursor-init
