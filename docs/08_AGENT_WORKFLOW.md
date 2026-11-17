# Multi-agent workflow — remote_viewer

Multiple Cursor agents (and possibly other tools) work in parallel. Conflict resistance
comes from **tasks + locks + worktrees + HANDOFF**.

Canonical short form also lives in [AGENTS.md](../AGENTS.md) and `.cursor/rules/40-multi-agent-protocol.mdc`.

## 1. Session start ritual (every agent, every session)

1. Read [AGENTS.md](../AGENTS.md)
2. Read [STATUS.md](STATUS.md)
3. Read last **5** entries of [HANDOFF.md](HANDOFF.md)
4. Read [07_TASKS.md](07_TASKS.md)
5. Read [agents/LOCKS.md](agents/LOCKS.md)
6. Read relevant ADRs + your [role file](agents/ROLES.md)

## 2. Claim before work

1. Pick a `TODO` task whose dependencies are `DONE`.
2. In **one small commit**: set `Status=CLAIMED`, `Owner=<agent-id>`, add a `LOCKS.md` row
   for every path in `Touches`, expiry default **24 h**.
3. If a path is locked by someone else → pick another task or set `BLOCKED` with reason.

## 3. One task = one branch = one PR

- Branch: `agent/<role>/<task-id>` (example: `agent/network-transport/T-0210`)
- Prefer `git worktree` so agents never share a working directory:

```bash
git worktree add ../remote_viewer-net -b agent/network-transport/T-0210
```

## 4. Stay inside `Touches`

- Need edits outside → new task or HANDOFF proposal; **do not** silently expand scope.
- Workspace `Cargo.toml` deps and `vd-proto` messages: **architect** only (or explicit task).

## 5. Definition of Done

All must hold:

- Compiles on Windows + Linux (cfg-gate as needed)
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo nextest run` (or `cargo test --workspace`) green
- Acceptance criteria met
- Docs updated if behavior/arch/protocol changed
- Perf-relevant → numbers in [05_PERFORMANCE_BUDGET.md](05_PERFORMANCE_BUDGET.md) results table
- Task → `REVIEW`; lock released; HANDOFF entry written

## 6. Reviewer marks DONE

Authors **never** mark their own task `DONE`. Reviewer verifies DoD, sets `DONE`, updates
`STATUS.md` + `CHANGELOG.md`.

## 7. HANDOFF entry format (append-only, newest at bottom)

```md
## 2026-11-17 14:30 · cursor-a1 · role: network-transport · T-0210
- Did: …
- State: (what works / what is stubbed)
- Decisions: (link ADR if any)
- Next: (specific next step)
- Gotchas: …
```

## 8. Never rewrite history of coordination

Do not delete/rewrite other agents’ HANDOFF entries or ADRs; **supersede** instead.

## 9. ADR required when…

Changing FIXED DIRECTION, heavy deps, wire protocol, crate boundaries, threading model.

## 10. Protocol changes

Bump `PROTOCOL_VERSION`, update [02_PROTOCOL.md](02_PROTOCOL.md), proptest roundtrip, compat notes.

## 11. Conflicts / uncertainty

Write under `## OPEN QUESTIONS` in HANDOFF; pick safest reversible option; continue.

## 12. Commit style

Conventional Commits + task id: `feat(vd-net): add pacer token bucket [T-0215]`

## 13. Honesty

Never mark stubbed work done; never invent bench numbers; say if tests could not run (no GPU).

## Example parallel wave (no file overlap)

| Role | Task area | Touches |
|------|-----------|---------|
| capture-codec | P1 loopback | `vd-capture`, `vd-codec` |
| network-transport | QUIC + frag on netsim | `vd-net` |
| rate-control | estimator + ladder | `vd-rate` |
| security-crypto | Noise + password | `vd-crypto` |
