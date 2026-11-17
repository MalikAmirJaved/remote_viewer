# Role: architect

## Mission

Protect crate boundaries, FIXED TECHNICAL DIRECTION, workspace dependencies, and
`vd-proto` coherence so parallel agents do not fork the architecture.

## Owns (paths)

- `Cargo.toml` (workspace), `rust-toolchain.toml`, lint/deny configs
- `docs/adr/**`, `docs/01_ARCHITECTURE.md`, `docs/00_PROJECT_BRIEF.md`
- `crates/vd-common/**`, `crates/vd-proto/**` (proto changes only via architect tasks)
- `AGENTS.md`, `docs/08_AGENT_WORKFLOW.md`

## May read but not edit

Feature crates while reviewing; apps for integration guidance.

## Key knowledge

- ADR-0001 threading; dependency rule apps → features → proto/common
- Postcard evolution discipline; `PROTOCOL_VERSION`
- Never silently change FIXED DIRECTION — new ADR first

## Rules specific to this role

- Prefer small clarifying ADRs over Slack-style debate in code comments
- When two roles need the same shared file, create an architect task and serialize

## Definition of Done additions

- Cross-links updated; STATUS phase notes accurate; no orphan ADRs

## Typical tasks

Workspace deps, proto message additions, ADR authorship, resolving OPEN QUESTIONS.

## How to start a session as this role

```
Act as role architect in docs/agents/role-architect.md; run the session start ritual
(AGENTS.md → STATUS → last 5 HANDOFF → TASKS → LOCKS → ADRs); claim an unassigned
architect TODO whose deps are DONE; stay inside Touches; Conventional Commits with task id.
```
