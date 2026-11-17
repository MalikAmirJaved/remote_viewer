# Roadmap — remote_viewer

Phases **P0–P9**. Each has a goal, major work, and **exit criteria**. Task details live in
[07_TASKS.md](07_TASKS.md).

## P0 — Foundation *(this init session)*

**Goal:** Compiling workspace, docs, agent system, proto, netsim, rate skeleton.  
**Exit:** `cargo fmt/clippy/test` green; docs/rules/roles present; P0 tasks DONE.

## P1 — Local pipeline

**Goal:** capture→encode→decode→render in **one process** (loopback); per-stage timing.  
**Tasks:** fake or real capture frames, openh264 path, wgpu/egui present frame, latency spans.  
**Exit:** Measured per-stage numbers recorded; loopback demo runs on Windows.

## P2 — LAN streaming

**Goal:** QUIC transport agent↔viewer on LAN; fragment/reassemble; basic input injection.  
**Exit:** Control a host over LAN; video + keyboard/mouse; no signal server required (direct).

## P3 — Internet connectivity

**Goal:** signal server, IDs, hole punch, relay fallback, Noise + password.  
**Exit:** Connect across NAT with consent UI; relay works when punch fails; E2E encrypted.

## P4 — Adaptive network

**Goal:** receiver reports, estimator, quality ladder live, pacer, FEC/NACK/IDR; all netem profiles.  
**Exit:** wan-weak profile meets usability targets; benches + netsim tests green.

## P5 — Hardware acceleration

**Goal:** NVENC/AMF/QSV encode, HW decode, zero-copy GPU, dirty-rect skipping.  
**Exit:** CPU budget met on reference HW machine; SW fallback still works.

## P6 — Feature parity

**Goal:** clipboard, file transfer, chat, audio, multi-monitor, client-side cursor, reconnect.  
**Exit:** Feature checklist signed off for Windows beta.

## P7 — Windows service & UAC

**Goal:** service + session helper; secure desktop capture/input; unattended access; tray indicator.  
**Exit:** UAC/secure desktop controllable with ethics rules intact; unattended opt-in only.

## P8 — Hardening

**Goal:** fuzz proto, security review, rate limits, abuse controls, opt-in crash reporting.  
**Exit:** Fuzz corpuses running in CI; security review notes addressed.

## P9 — Packaging & release

**Goal:** MSI/NSIS, signed auto-update, Linux packages, docs, beta.  
**Exit:** Installable beta with update channel; public docs.

## Related

[07_TASKS.md](07_TASKS.md) · [STATUS.md](STATUS.md) · [10_RISKS.md](10_RISKS.md)
