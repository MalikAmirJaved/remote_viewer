# 0005 — Capture / encode / decode traits

- **Status:** Accepted
- **Date:** 2026-11-17
- **Deciders:** cursor-init (architect)

## Context

Platforms differ wildly (DXGI, PipeWire, ScreenCaptureKit; NVENC/AMF/QSV/openh264).
We need one pipeline with swappable backends.

## Decision

- Traits: `Capturer`, `Encoder`, `Decoder` in `vd-capture` / `vd-codec`.
- Windows capture: **DXGI Desktop Duplication** primary, **WGC** fallback.
- Encode: H.264 first — HW when available, **openh264** SW fallback.
- Low-latency encode config: no B-frames, no lookahead, CBR/tight VBV, infinite GOP +
  on-demand IDR + intra-refresh.
- Decode: HW (D3D11VA/MF) with SW fallback; render via **wgpu** (viewer).
- P1 proves loopback with SW; P5 adds HW / zero-copy.

## Consequences

### Positive

- Parallel agents can implement backends without rewriting the pipeline.
- Testability via stub/fake frames.

### Negative / risks

- Trait objects may inhibit some zero-copy optimizations — use associated types or generics
  at hot boundaries later if needed.

## Alternatives considered

1. **FFmpeg as the only backend** — heavier licensing/build story; avoid as hard dependency.
2. **Per-platform monolithic pipelines** — duplicates rate-control and net logic.
