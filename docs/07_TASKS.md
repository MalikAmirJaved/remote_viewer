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
| T-0101 | Fake frame source for loopback | P1 | REVIEW | opencode-a1 | T-0006 |
| T-0102 | Wire openh264 SW encoder | P1 | REVIEW | opencode-a1 | T-0101 |
| T-0103 | Wire openh264 SW decoder | P1 | REVIEW | opencode-a1 | T-0102 |
| T-0104 | Loopback pipeline thread + channels | P1 | CLAIMED | opencode-a1 | T-0103 |
| T-0105 | egui present decoded texture | P1 | TODO | unassigned | T-0104 |
| T-0106 | Per-stage latency spans + report | P1 | TODO | unassigned | T-0104 |
| T-0107 | DXGI capturer MVP (Windows) | P1 | TODO | unassigned | T-0006 |
| T-0108 | P1 dependency unblock + ADR-0007 | P1 | REVIEW | opencode-a1 | T-0006 |
| T-0201 | Transport trait integration tests w/ netsim | P2 | REVIEW | opencode-a1 | T-0004 |
| T-0202 | Quinn connect/accept wrapper | P2 | TODO | unassigned | T-0201 |
| T-0203 | Reliable streams mapping for Control/Input | P2 | TODO | unassigned | T-0202 |
| T-0204 | Datagram send/recv for Video/Feedback | P2 | TODO | unassigned | T-0202 |
| T-0205 | Video fragmenter (~1150 B) | P2 | TODO | unassigned | T-0003 |
| T-0206 | Video reassembler latest-frame-wins | P2 | TODO | unassigned | T-0205 |
| T-0207 | Token-bucket pacer | P2 | TODO | unassigned | T-0204 |
| T-0208 | Agent↔viewer LAN demo binary flags | P2 | TODO | unassigned | T-0203,T-0206,T-0104 |
| T-0209 | SendInput keyboard/mouse MVP | P2 | TODO | unassigned | T-0006 |
| T-0210 | InputMove coalesce ≤125 Hz | P2 | TODO | unassigned | T-0204,T-0209 |
| T-0301 | Signal: register peer ID (SQLite) | P3 | TODO | unassigned | T-0008 |
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
- **Status:** REVIEW
- **Phase:** P1
- **Role:** capture-codec
- **Owner:** opencode-a1
- **Depends on:** T-0006
- **Touches:** crates/vd-capture/**
- **Branch:** agent/capture-codec/T-0101
- **Acceptance:**
  - `Capturer` impl yields patterned BGRA frames at configurable fps; unit test
- **Notes / links:** docs/06_ROADMAP.md#p1--local-pipeline
- **Updated:** 2026-11-18 by opencode-a1

### T-0102 · Wire openh264 SW encoder
- **Status:** REVIEW
- **Phase:** P1
- **Role:** capture-codec
- **Owner:** opencode-a1
- **Depends on:** T-0101
- **Touches:** crates/vd-codec/**
- **Branch:** agent/capture-codec/T-0102
- **Acceptance:**
  - Encode fake frame to Annex-B/AVCC H.264; force_idr works; low-latency flags set
- **Notes / links:** docs/adr/0005-capture-encode-decode-traits.md
- **Updated:** 2026-11-18 by opencode-a1

### T-0103 · Wire openh264 SW decoder
- **Status:** REVIEW
- **Phase:** P1
- **Role:** capture-codec
- **Owner:** opencode-a1
- **Depends on:** T-0102
- **Touches:** crates/vd-codec/**
- **Branch:** agent/capture-codec/T-0103
- **Acceptance:**
  - Decode encoder output to BGRA/YUV; roundtrip test on fake content
- **Notes / links:** output is RGBA8, not BGRA — see docs/HANDOFF.md 2026-11-18
- **Updated:** 2026-11-18 by opencode-a1

### T-0104 · Loopback pipeline thread + channels
- **Status:** CLAIMED
- **Phase:** P1
- **Role:** architect
- **Owner:** opencode-a1
- **Depends on:** T-0103
- **Touches:** apps/vd-agent/**, apps/vd-viewer/** OR new tools/loopback
- **Branch:** agent/architect/T-0104
- **Acceptance:**
  - Dedicated threads + crossbeam caps 1–2; drop-oldest demonstrated under overload test
- **Notes / links:** docs/01_ARCHITECTURE.md
- **Updated:** 2026-11-17 by cursor-init

### T-0105 · egui present decoded texture
- **Status:** TODO
- **Phase:** P1
- **Role:** viewer-ui
- **Owner:** unassigned
- **Depends on:** T-0104
- **Touches:** apps/vd-viewer/**
- **Branch:** agent/viewer-ui/T-0105
- **Acceptance:**
  - Window shows loopback frames via egui/wgpu; close cleanly
- **Notes / links:** docs/adr/0006-viewer-ui-egui.md
- **Updated:** 2026-11-17 by cursor-init

### T-0106 · Per-stage latency spans + report
- **Status:** TODO
- **Phase:** P1
- **Role:** qa-bench
- **Owner:** unassigned
- **Depends on:** T-0104
- **Touches:** tools/vd-bench/**, docs/05_PERFORMANCE_BUDGET.md
- **Branch:** agent/qa-bench/T-0106
- **Acceptance:**
  - p50 per stage printed; results row added (or HANDOFF if env blocked)
- **Notes / links:** docs/05_PERFORMANCE_BUDGET.md
- **Updated:** 2026-11-17 by cursor-init

### T-0107 · DXGI capturer MVP (Windows)
- **Status:** TODO
- **Phase:** P1
- **Role:** capture-codec
- **Owner:** unassigned
- **Depends on:** T-0006
- **Touches:** crates/vd-capture/**
- **Branch:** agent/capture-codec/T-0107
- **Acceptance:**
  - Capture primary monitor; handle access-lost reinit; cfg-gated so Linux still builds
- **Notes / links:** docs/agents/role-capture-codec.md
- **Updated:** 2026-11-17 by cursor-init

### T-0108 · P1 dependency unblock + ADR-0007
- **Status:** REVIEW
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
- **Notes / links:** docs/adr/0007-openh264-sw-codec.md, docs/12_DEV_SETUP.md
- **Updated:** 2026-11-18 by opencode-a1
### T-0201 · Transport trait integration tests w/ netsim
- **Status:** REVIEW
- **Phase:** P2
- **Role:** network-transport
- **Owner:** opencode-a1
- **Depends on:** T-0004
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0201
- **Acceptance:**
  - In-memory `Transport` adapter over SimLink; send/recv test
- **Notes / links:** docs/adr/0002-transport-quic-over-udp.md
- **Updated:** 2026-11-18 by opencode-a1

### T-0202 · Quinn connect/accept wrapper
- **Status:** TODO
- **Phase:** P2
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0201
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0202
- **Acceptance:**
  - LAN connect between two processes; idle timeout configured; docs for knobs in HANDOFF
- **Notes / links:** —
- **Updated:** 2026-11-17 by cursor-init

### T-0203 · Reliable streams mapping for Control/Input
- **Status:** TODO
- **Phase:** P2
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0202
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0203
- **Acceptance:**
  - Control + Input on separate streams; Input marked high priority if API allows
- **Notes / links:** docs/02_PROTOCOL.md
- **Updated:** 2026-11-17 by cursor-init

### T-0204 · Datagram send/recv for Video/Feedback
- **Status:** TODO
- **Phase:** P2
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0202
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0204
- **Acceptance:**
  - Datagram path works; oversized payloads rejected cleanly
- **Notes / links:** —
- **Updated:** 2026-11-17 by cursor-init

### T-0205 · Video fragmenter (~1150 B)
- **Status:** TODO
- **Phase:** P2
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0003
- **Touches:** crates/vd-net/** or crates/vd-proto/**
- **Branch:** agent/network-transport/T-0205
- **Acceptance:**
  - Splits EncodedFrame into VideoFrameFragment list; unit test
- **Notes / links:** docs/03_NETWORK_ADAPTATION.md
- **Updated:** 2026-11-17 by cursor-init

### T-0206 · Video reassembler latest-frame-wins
- **Status:** TODO
- **Phase:** P2
- **Role:** network-transport
- **Owner:** unassigned
- **Depends on:** T-0205
- **Touches:** crates/vd-net/**
- **Branch:** agent/network-transport/T-0206
- **Acceptance:**
  - Completes frames; drops incomplete older frames when newer completes; loss test via netsim
- **Notes / links:** —
- **Updated:** 2026-11-17 by cursor-init

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
- **Status:** TODO
- **Phase:** P2
- **Role:** input-platform
- **Owner:** unassigned
- **Depends on:** T-0006
- **Touches:** crates/vd-input/**
- **Branch:** agent/input-platform/T-0209
- **Acceptance:**
  - Key/button/move inject on Windows; `release_all` clears; Linux cfg stub remains
- **Notes / links:** docs/agents/role-input-platform.md
- **Updated:** 2026-11-17 by cursor-init

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
