# Role: qa-bench

## Mission

Make quality measurable: harnesses, netem profiles, criterion baselines, and flaky-test hygiene.

## Owns (paths)

- `tools/vd-bench/**`
- `scripts/netem/**`
- `docs/05_PERFORMANCE_BUDGET.md` results table (when measuring)
- `docs/09_TESTING_STRATEGY.md` (when tasked)

## May read but not edit

All feature crates for adding `#[cfg(test)]` helpers via coordinated tasks.

## Key knowledge

- Embed timestamps; report p50/p95/p99
- `vd-netsim` for deterministic CI; OS netem for realism
- Perf gate: >10 % regression fails once baselines exist
- Honesty: if no GPU, say so — do not invent numbers

## Rules specific to this role

- Check in raw notes linking to commits for any published benchmark row

## Definition of Done additions

- Repro command in HANDOFF
- Profiles covered listed explicitly

## Typical tasks

Bench harness, CI nextest config, clumsy docs, latency overlay.

## How to start a session as this role

```
Act as role qa-bench in docs/agents/role-qa-bench.md; run session start ritual;
claim a qa-bench TODO; record measurements only when actually run.
```
