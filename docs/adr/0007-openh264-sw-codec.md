# 0007 — Software H.264 codec binding: `openh264` crate

- **Status:** Accepted
- **Date:** 2026-11-18
- **Deciders:** opencode-a1 (architect)

## Context

P1 needs a real software H.264 encoder and decoder so the loopback pipeline
(capture → encode → decode → present) can be proven end-to-end without a GPU.
ADR-0005 already fixed the architecture: H.264 first, **openh264** as the software
fallback behind the `Encoder` / `Decoder` traits in `vd-codec`.

What ADR-0005 did not fix is *how* openh264 enters the build. The decision matters
because it changes developer prerequisites, CI cost, and the licensing story:

- The window `cargo add` publishes several bindings with very different build models.
- `crates/vd-codec` must stay compilable on Linux and Windows (CI matrix does both).
- `docs/12_DEV_SETUP.md` promises "FFmpeg-free by default"; the chosen binding must
  honour that promise.

## Decision

Use the **`openh264` crate (0.9.10, BSD-2-Clause, MSRV 1.85)** with its default
`source` feature, pinned in `[workspace.dependencies]` and wired into
`crates/vd-codec`.

- The crate vendors Cisco's OpenH264 C sources and compiles them via the `cc` crate
  (`openh264-sys2`). **No runtime download**, no opaque prebuilt binary.
- Consequence for the environment: a **C toolchain is now a prerequisite** —
  MSVC Build Tools on Windows, `build-essential` on Linux. Both GitHub-hosted runner
  images already provide one, so CI needs no extra step; contributors do.
- Licence: BSD-2-Clause (source), matching the upstream Cisco licence. OpenH264
  binary patent-licence terms only bite if we **ship** prebuilt binaries, which is a
  P9 packaging question and is recorded in `docs/10_RISKS.md`.
- The binding is pinned at the workspace level, not per crate, so P5 (NVENC) and any
  later HEVC/AV1 backend are added or removed in one place.
- eframe/egui are **not** covered by this ADR: ADR-0006 already fixed egui/eframe, so
  their version pins live in `[workspace.dependencies]` without a new ADR. The
  `glow` renderer is used for the P1/MVP path; `wgpu` stays available for the
  zero-copy P5 path (`T-0502`).

## Consequences

### Positive

- Build is reproducible and auditable: the C source we compile is the source we vendor.
- No FFmpeg, no proprietary SDK redistribution, no runtime fetch on first encode.
- `rust-version` of the binding (1.85) equals the workspace MSRV, so no bump.

### Negative / risks

- Every fresh checkout and every CI job pays the C compile cost of `openh264-sys2`
  (tens of seconds), on top of eframe's build time.
- A missing/misconfigured C toolchain now fails `cargo check` for **all** workspace
  members, not just the codec — surfaced deliberately in the unblock commit (T-0108)
  so it fails on an architect branch rather than mid-implementation.
- OpenH264's own codec quality/bitrate behaviour on the low-latency settings
  (no B-frames, infinite GOP, on-demand IDR) is unverified until T-0102 measures it.

## Alternatives considered

1. **`shiguredo_openh264` (2026.2.0)** — ships prebuilt binaries; simplest build, but
   pushes an opaque blob into our supply chain and complicates the P9 signed-release
   story. Rejected for now; the `libloading`-style split remains a fallback if C
   toolchain friction becomes a real problem (would be a new ADR).
2. **FFmpeg bindings** — widest codec coverage, heaviest build/licence story, and
   explicitly out per `docs/12_DEV_SETUP.md`. Rejected.
3. **Pure-Rust H.264 (`h26x-extractor` / `rust_h264` style)** — attractive for a
   toolchain-free build, but no mature low-latency encoder exists; would put P1 at
   risk. Rejected for P1.
4. **Media Foundation (Windows-only)** — good long-term host-side option, but it would
   make the P1 loopback untestable on Linux and is a P5 concern, not P1.

## Related

[0005](0005-capture-encode-decode-traits.md) · [0006](0006-viewer-ui-egui.md) ·
[docs/12_DEV_SETUP.md](../12_DEV_SETUP.md) · [docs/10_RISKS.md](../10_RISKS.md)
