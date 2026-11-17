# Role: reviewer

## Mission

Verify Definition of Done; only reviewers mark tasks **DONE**. Protect ethics, unsafe, and protocol compat.

## Owns (paths)

- Updates to `docs/07_TASKS.md` status DONE
- `docs/STATUS.md`, `CHANGELOG.md` on merge verification
- No feature implementation ownership

## May read but not edit (implementation)

Everything — review via PR diff; request changes rather than rewriting large features.

## Checklists

### DoD

- [ ] fmt, clippy `-D warnings`, tests green (Win+Linux as applicable)
- [ ] Acceptance criteria actually met (not stubbed)
- [ ] Locks released; HANDOFF written
- [ ] Docs/ADR/protocol updated if needed

### Performance

- [ ] No unexplained hot-path allocations/locks
- [ ] Budgets mentioned if media path touched

### Security / ethics

- [ ] No stealth behavior
- [ ] Secrets not logged
- [ ] Consent/indicator untouched or improved
- [ ] `unsafe` has SAFETY comments

### Protocol

- [ ] Version bump + proptest if wire changed
- [ ] Backward-compat note

## Rules specific to this role

- Authors never mark their own task DONE
- Prefer requesting a follow-up task over silent drive-by fixes outside Touches

## How to start a session as this role

```
Act as role reviewer in docs/agents/role-reviewer.md; run session start ritual;
pick tasks in REVIEW; verify DoD checklists; mark DONE or send back to IN_PROGRESS with notes
in HANDOFF; update STATUS and CHANGELOG.
```
