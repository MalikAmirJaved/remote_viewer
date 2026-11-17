# AGENTS.md — remote_viewer

Instructions for **every** AI coding agent working in this repository.

## Product

**remote_viewer** is a clean-room, high-performance remote desktop (ID + password, screen,
input, files, chat). Internal crates use the `vd-*` prefix. Read
[docs/00_PROJECT_BRIEF.md](docs/00_PROJECT_BRIEF.md) for goals and hard ethics rules.

## Session start ritual (mandatory)

1. This file (`AGENTS.md`)
2. [docs/STATUS.md](docs/STATUS.md)
3. Last **5** entries of [docs/HANDOFF.md](docs/HANDOFF.md)
4. [docs/07_TASKS.md](docs/07_TASKS.md)
5. [docs/agents/LOCKS.md](docs/agents/LOCKS.md)
6. Relevant [ADRs](docs/adr/README.md) + your [role file](docs/agents/ROLES.md)

Full protocol: [docs/08_AGENT_WORKFLOW.md](docs/08_AGENT_WORKFLOW.md).

## Claiming work

- Pick `TODO` with dependencies `DONE`.
- One commit: `CLAIMED` + Owner + `LOCKS.md` row (24 h) for all `Touches` paths.
- Branch: `agent/<role>/<task-id>`; prefer **git worktree** per agent.
- Stay inside `Touches`. Shared workspace/`vd-proto` edits = **architect** (or explicit task).

## Definition of Done

`fmt` · `clippy -D warnings` · tests green · acceptance met · docs if needed ·
Status `REVIEW` · lock released · HANDOFF appended. **Reviewer** marks `DONE`.

## Commits

Conventional Commits with task id, e.g. `feat(vd-net): add pacer token bucket [T-0207]`.

## Ethics (non-negotiable)

Visible host indicator; host can disconnect; unattended = explicit opt-in; **no stealth RAT
behavior**.

## Honesty

Never mark stubs as done; never invent benchmark numbers; append-only HANDOFF/ADRs.
