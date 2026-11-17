# Architecture — remote_viewer

## 1. Component diagram

```mermaid
flowchart LR
  subgraph Host
    Agent[vd-agent]
    Cap[vd-capture]
    Enc[vd-codec Encoder]
    Inj[vd-input]
    Rate[vd-rate]
    Agent --> Cap --> Enc
    Agent --> Inj
    Agent --> Rate
  end

  subgraph Network
    Net[vd-net Transport]
    Punch[Hole punch]
    Relay[vd-relay]
    Signal[vd-signal]
  end

  subgraph Client
    Viewer[vd-viewer]
    Dec[vd-codec Decoder]
    UI[egui/wgpu]
    Viewer --> Dec --> UI
  end

  Agent <--> Net
  Viewer <--> Net
  Net --> Punch
  Net -.-> Relay
  Agent <---> Signal
  Viewer <---> Signal
  Crypto[vd-crypto Noise] --- Agent
  Crypto --- Viewer
```

## 2. Hot-path data flow and thread boundaries

### Host: capture → encode → send

```mermaid
flowchart TB
  TCap[OS thread: capture] -->|channel cap 2 drop-oldest| TEnc[OS thread: encode]
  TEnc -->|channel cap 2| TPace[OS thread: packetize + pacer]
  TPace -->|datagrams| UDP[Transport UDP/QUIC]
  TInp[OS thread or async: input recv] --> Inj[InputInjector]
  TFb[Feedback recv ~20Hz] --> Rate[vd-rate]
  Rate -->|bitrate/fps/scale/IDR| TEnc
```

Queue policy: **latest-frame-wins**. Never block capture on encode backlog.

### Viewer: recv → decode → render

```mermaid
flowchart TB
  UDP[Transport] --> TAsm[OS thread: reassembly]
  TAsm -->|channel cap 1-2| TDec[OS thread: decode]
  TDec -->|channel cap 1| TRen[OS thread / UI: wgpu render]
  UI[egui input] --> SendIn[Input / InputMove send]
```

Jitter buffer: **0–1 frame**. Render ASAP; no smoothing delay in v1.

## 3. Crate dependency graph

**Rule:** dependencies only point downward:

`apps/*` → feature crates → `vd-proto` / `vd-common`

```
apps: vd-agent, vd-viewer, vd-signal, vd-relay
tools: vd-bench, xtask

feature: vd-net, vd-capture, vd-codec, vd-rate, vd-input,
         vd-audio, vd-clipboard, vd-transfer, vd-crypto, vd-netsim

leaf: vd-proto → vd-common
```

Shared files (`Cargo.toml` workspace deps, `vd-proto` messages) are edited only by
**architect** or an explicitly assigned task — others request via [HANDOFF.md](HANDOFF.md).

## 4. Per-stage latency budget *(initial targets, to be validated)*

| Stage | Budget |
|-------|--------|
| Capture | ≤ 4 ms |
| Encode | ≤ 8 ms |
| Packetize + pace | ≤ 2 ms |
| Network | ≈ RTT/2 |
| Reassembly | ≤ 1 ms |
| Decode | ≤ 6 ms |
| Render (incl. vsync strategy) | ≤ 8 ms |

LAN glass-to-glass p50 ≤ 40 ms ⇒ non-network stages ~≤ 29 ms combined on the happy path.

Measurement method: [05_PERFORMANCE_BUDGET.md](05_PERFORMANCE_BUDGET.md).

## 5. Threading model

See [ADR-0001](adr/0001-rust-workspace-and-threading-model.md).

- Media hot paths: dedicated OS threads, no `.await`.
- Bounded `crossbeam-channel` between stages.
- Tokio: signaling HTTP/WS, relay event loop, connection setup/teardown only.
- Never run encode/decode on the tokio reactor thread.

## 6. Memory strategy

- Buffer pools for capture/encode scratch; prefer `bytes::Bytes` for wire buffers.
- **No per-frame heap allocation in steady state** (goal; enforce in P1/P5 with benches).
- Zero-copy GPU texture upload when HW path allows (P5).
- Cap clipboard/file chunk sizes; reject oversized framed messages.

## 7. Platform abstraction

Traits: `Capturer`, `Encoder`, `Decoder`, `Transport`, `InputInjector`, `RateControl`, `KeyStore`,
`ClipboardSync`. Platform modules are `cfg`-gated. Windows is the first production target.

## 8. Failure modes and recovery

| Failure | Recovery |
|---------|----------|
| Capture access lost (desktop switch / resolution) | Re-init capturer; force IDR |
| Encoder error | Restart encoder; IDR; optional SW fallback |
| Packet loss | NACK small gaps; IDR / intra-refresh; FEC on keyframes |
| Path dead | Probe direct vs relay; migrate; resume with IDR |
| Auth / consent revoke | Tear down immediately; release keys |
| Input disconnect | `release_all()` stuck keys |

## 9. Extension points

- Web viewer via WebTransport later (same `Transport` semantics).
- Mobile: separate apps consuming `vd-proto` / `vd-net`.
- Session recording: tap encoded bitstream (consent + indicator required).
- Alt transport: `str0m` WebRTC behind `Transport` ([ADR-0002](adr/0002-transport-quic-over-udp.md)).

## Related

[02_PROTOCOL.md](02_PROTOCOL.md) · [03_NETWORK_ADAPTATION.md](03_NETWORK_ADAPTATION.md) · [04_SECURITY.md](04_SECURITY.md)
