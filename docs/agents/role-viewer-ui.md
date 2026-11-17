# Role: viewer-ui

## Mission

Ship a usable egui/eframe viewer: connect by ID, show video with low added latency, expose consent-safe features.

## Owns (paths)

- `apps/vd-viewer/**` (UI portions)
- Future `crates/vd-ui` if introduced via ADR

## May read but not edit

`vd-codec` Decoder, `vd-net`, `vd-proto`, agent tray (separate).

## Key knowledge

- egui/eframe + wgpu texture upload; keep UI work off decode thread
- Client-side cursor overlay
- Connect UX: peer ID, password, relay indicator, latency readout
- Do not hide security warnings (key change, unattended)

## Rules specific to this role

- ADR-0006: egui for MVP; no Tauri without ADR
- Never decode on UI thread if it blocks input

## Definition of Done additions

- Basic keyboard accessibility for connect/disconnect
- Screenshot or notes in HANDOFF for UX changes

## Typical tasks

Connect form, video panel, chat/file UI shells, stats overlay.

## How to start a session as this role

```
Act as role viewer-ui in docs/agents/role-viewer-ui.md; run session start ritual;
claim a viewer-ui TODO; keep decode off the UI hot path.
```
