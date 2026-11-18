# Dev setup — remote_viewer

## Prerequisites

- **Rust**: toolchain from `rust-toolchain.toml` (rustup will install).
- **OS**: Windows 10/11 recommended for host capture work; Linux OK for most crates.
- **Git** worktrees recommended for multi-agent work.

### Windows

- **C toolchain: MSVC Build Tools** with the C++ workload — required, not optional,
  since [ADR-0007](adr/0007-openh264-sw-codec.md) compiles Cisco's OpenH264 C source.
  `cl.exe` does not need to be on `PATH`; `cc` locates the MSVC install via the registry.
- Optional: GPU vendor SDK for NVENC/AMF/QSV (**not required for P0–P1 SW path**).
- Optional: [clumsy](https://jagt.github.io/clumsy/) for netem-like shaping.

### Linux

- `build-essential` (**required** for the openh264 C build, see
  [ADR-0007](adr/0007-openh264-sw-codec.md)), `pkg-config`.
- Optional: PipeWire dev headers when implementing Linux capture.
- `iproute2` (`tc`) for netem scripts (root/`NET_ADMIN`).

### macOS

- Xcode CLT (provides the C compiler openh264 needs); ScreenCaptureKit when
  implementing macOS capture.

## FFmpeg-free by default

P0–P1 builds **do not** require FFmpeg. Prefer Media Foundation / vendor encode APIs /
`openh264` crates. If FFmpeg is introduced later, it needs an ADR and clear license notes.

## Quickstart

```bash
cargo xtask check
# or:
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Bin stubs:

```bash
cargo run -p vd-agent
cargo run -p vd-viewer
cargo run -p vd-signal
cargo run -p vd-relay
cargo run -p vd-bench -- --profile lan
```

## Codec SDK notes

| Backend | When | Notes |
|---------|------|-------|
| openh264 | P1 | SW reference path |
| NVENC | P5 | NVIDIA GPU; driver-dependent |
| AMF | P5 | AMD |
| QuickSync | P5 | Intel via MF/QSV |
| D3D11VA / MF decode | P5 | Windows HW decode |

Do not commit proprietary SDK redistributables without legal clearance (P9).

## Related

[AGENTS.md](../AGENTS.md) · [09_TESTING_STRATEGY.md](09_TESTING_STRATEGY.md)
