# Agent roles — remote_viewer

Pick the role that matches the task’s `Role:` field. Read that role file before claiming work.

| Role | File | Use when |
|------|------|----------|
| architect | [role-architect.md](role-architect.md) | Workspace deps, crate boundaries, ADRs, proto ownership |
| capture-codec | [role-capture-codec.md](role-capture-codec.md) | DXGI/WGC, encoders/decoders, GPU buffers |
| network-transport | [role-network-transport.md](role-network-transport.md) | quinn, punch, relay client, pacer, framing |
| rate-control | [role-rate-control.md](role-rate-control.md) | Estimator, ladder, FEC ratio, netsim traces |
| security-crypto | [role-security-crypto.md](role-security-crypto.md) | Noise, keys, password, consent crypto aspects |
| input-platform | [role-input-platform.md](role-input-platform.md) | SendInput, scancodes, UAC helper later |
| viewer-ui | [role-viewer-ui.md](role-viewer-ui.md) | egui/eframe, textures, connect UX |
| server-signal-relay | [role-server-signal-relay.md](role-server-signal-relay.md) | axum signal, UDP relay |
| qa-bench | [role-qa-bench.md](role-qa-bench.md) | Harness, netem, criterion gates |
| reviewer | [role-reviewer.md](role-reviewer.md) | DoD verification; marks DONE |

Active path claims: [LOCKS.md](LOCKS.md)
