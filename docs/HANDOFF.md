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
