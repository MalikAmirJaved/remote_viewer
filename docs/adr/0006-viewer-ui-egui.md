# 0006 — Viewer UI: egui / eframe

- **Status:** Accepted
- **Date:** 2026-11-17
- **Deciders:** cursor-init (architect)

## Context

MVP needs a native viewer quickly on Windows/Linux/macOS without a web stack (web viewer
is a non-goal for v1).

## Decision

- Use **`egui` / `eframe`** for the MVP viewer UI (connect form, video surface, chat, transfers).
- Video presented via egui/wgpu texture path; revisit **Slint** or **Tauri** only via new ADR
  if egui blocks performance or packaging goals.

## Consequences

### Positive

- Fast iteration in Rust; good enough for internal beta.

### Negative / risks

- Immediate-mode UI may fight 60 fps remote video if not careful — keep UI work off the
  decode/render hot path where possible.

## Alternatives considered

1. **Tauri + web UI** — rejected for v1 (web viewer non-goal; more attack surface).
2. **Iced / Slint** — viable later; egui chosen for speed to first pixels.
