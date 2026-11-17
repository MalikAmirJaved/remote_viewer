# Testing strategy — remote_viewer

## Layers

| Layer | Tooling | Owns |
|-------|---------|------|
| Unit | `cargo test` / `cargo nextest` | pure logic: rate, proto, netsim, transfer |
| Property | `proptest` | proto roundtrip, fragment invariants |
| Bench | `criterion` + `vd-bench` | encode envelope, rate, e2e latency |
| In-process net | `vd-netsim` | loss/latency/bw/reorder deterministic traces |
| OS shaping | `tc netem` / clumsy | E2E realism |
| Fuzz | `cargo fuzz` (P8) | parsers in `vd-proto` |

## Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace   # or: cargo test --workspace
cargo xtask check
```

## Mandatory per PR

- Unit/property tests for touched logic
- No `todo!()` on paths executed by tests — use `Error::NotImplemented`
- Windows + Linux CI (cfg-gate platform FFI)

## Network profiles

See [05_PERFORMANCE_BUDGET.md](05_PERFORMANCE_BUDGET.md) and [`scripts/netem/`](../scripts/netem).

P4 exit requires automated checks under **all** named profiles (netsim and/or OS shaping).

## Perf regressions

- Baseline numbers live in the performance budget results table.
- CI gate: fail if criterion metrics regress **> 10 %** once baselines exist.

## Security tests

- Proto decode of truncated/oversized buffers must fail cleanly.
- Crypto tests use `MemoryKeyStore`; never commit real keys.

## Related

[07_TASKS.md](07_TASKS.md) · role [agents/role-qa-bench.md](agents/role-qa-bench.md)
