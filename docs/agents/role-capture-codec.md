# Role: capture-codec

## Mission

Deliver a low-latency capture→encode→decode path with correct platform semantics and
no silent CPU readbacks on the HW path.

## Owns (paths)

- `crates/vd-capture/**`
- `crates/vd-codec/**`

## May read but not edit

`vd-rate` (bitrate APIs), `vd-proto` video types, `vd-common` clock/errors, viewer render glue.

## Key knowledge / pitfalls

- **DXGI Desktop Duplication:** `DXGI_ERROR_ACCESS_LOST` on desktop switch / resolution /
  secure desktop — re-create duplication output; force IDR after recovery.
- Prefer **dirty/move rects**; skip encode when empty; cursor is often **separate** from frame.
- Pool **D3D11 textures**; convert to NV12 on GPU when feeding HW encoders.
- Encoder low-latency flags: no B-frames, no lookahead, tight VBV/CBR, infinite GOP,
  on-demand IDR, intra-refresh where supported (NVENC/AMF/QSV differ — document per backend).
- Avoid CPU readback in steady state; SW openh264 path may copy BGRA for P1 only.
- Each `unsafe` needs `// SAFETY:` explaining invariants.

## Rules specific to this role

- No `.await` in capture/encode loops
- No allocations in per-frame steady state without a measured justification
- Logging above `debug` forbidden in per-frame code

## Definition of Done additions

- Access-lost recovery tested or HANDOFF why not
- Stage timings under budget or explicitly flagged

## Typical tasks

Stub→DXGI, openh264 wire-up, HW encoder wrappers, dirty-rect skip.

## How to start a session as this role

```
Act as role capture-codec in docs/agents/role-capture-codec.md; run the session start ritual;
claim a capture-codec TODO (P1 loopback preferred); lock vd-capture/vd-codec only; no proto edits
without architect task.
```
