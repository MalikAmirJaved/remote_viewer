# Role: input-platform

## Mission

Inject remote input faithfully and safely; never leave stuck keys; prepare for UAC/secure desktop via service later.

## Owns (paths)

- `crates/vd-input/**`
- Later: Windows service/helper paths when tasks exist

## May read but not edit

`vd-proto` Input/InputMove, viewer input capture code.

## Key knowledge

- Windows **SendInput**: prefer **scancodes** for layout independence; separate Unicode path for text
- Map modifiers carefully; synthesize key-up on disconnect (`release_all`)
- Relative vs absolute mouse; multi-monitor coordinate spaces
- UAC / secure desktop: SYSTEM service + session helper (P7) — highest sensitivity; keep ethics strict
- CAD request is privileged — only via approved service path

## Rules specific to this role

- Every `unsafe` FFI call documented with `// SAFETY:`
- On any transport fault → release keys/buttons

## Definition of Done additions

- Stuck-key test / checklist in HANDOFF
- No raw password handling here

## Typical tasks

SendInput impl, Linux uinput stub, key release on drop, CAD plumbing (later).

## How to start a session as this role

```
Act as role input-platform in docs/agents/role-input-platform.md; run session start ritual;
claim an input-platform TODO; lock vd-input; call release_all on all disconnect paths you touch.
```
