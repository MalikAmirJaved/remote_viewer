# Role: rate-control

## Mission

Keep sessions usable on terrible networks via closed-loop estimation, ladder hysteresis,
and FEC ratio — pure logic first, then wire to Feedback channel.

## Owns (paths)

- `crates/vd-rate/**`

## May read but not edit

`vd-netsim`, `vd-proto` Feedback types (request changes via HANDOFF/architect), `vd-codec` bitrate APIs.

## Key knowledge

- Delay-gradient (Kalman/trendline) + loss-based signals; AIMD/BBR-inspired ideas
- Ladder hysteresis: fast down (~100 ms), slow up (~2–3 s)
- text-sharp vs motion mode bias
- FEC ratio mapping from `loss_ppm`
- Drive tests with **deterministic** `vd-netsim` traces — no flaky timing tests

## Rules specific to this role

- No IO in `vd-rate` (keep crate pure)
- Never invent “improved” latency numbers without benches

## Definition of Done additions

- Unit tests for down / hold / up (extend beyond skeleton)
- Trace fixtures checked in under `vd-rate` or `vd-netsim` tests

## Typical tasks

Estimator, ladder bias, FEC controller, integration with agent encode settings.

## How to start a session as this role

```
Act as role rate-control in docs/agents/role-rate-control.md; run session start ritual;
claim a rate-control TODO; keep vd-rate IO-free; use vd-netsim for traces.
```
