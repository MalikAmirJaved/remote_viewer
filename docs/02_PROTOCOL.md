# Protocol — remote_viewer

**Version:** `PROTOCOL_VERSION = 1` ([`vd-proto`](../crates/vd-proto))  
**Serialization:** serde + postcard ([ADR-0003](adr/0003-serialization-postcard.md))

## Envelope

```text
Envelope {
  version: u16,      // must match negotiated version
  channel: Channel,  // multiplex key
  seq:     u64,      // per-channel sequence
  ts_us:   u64,      // sender monotonic µs
  payload: Payload,  // channel-specific
}
```

## Channels

| Channel | Delivery | Content |
|---------|----------|---------|
| `Control` | reliable, ordered | Hello/Auth/Capabilities, session start/stop, quality hints, monitors, ping |
| `Input` | reliable, ordered, **high priority** | key down/up, buttons, wheel, text, CAD request |
| `InputMove` | unreliable datagram, latest-wins | mouse position (coalesced ≤ 125 Hz) |
| `Video` | unreliable datagram | `VideoFrameFragment` |
| `Cursor` | reliable | cursor shape (cached by id) + hotspot |
| `Audio` | unreliable datagram | Opus frames |
| `Feedback` | unreliable datagram ~20 Hz | `ReceiverReport`, `Nack` |
| `Clipboard` | reliable | typed payloads + consent flag |
| `File` | reliable per transfer | offer/accept/chunk/ack/resume/cancel + BLAKE3 |
| `Chat` | reliable | text |

### VideoFrameFragment

`{ frame_id, frag_idx, frag_total, is_keyframe, capture_ts_us, codec, width, height, dirty_rects?, data }`

Fragments are MTU-safe (~1150 B data). Reassembly drops incomplete old frames when a newer
`frame_id` completes (latest-frame-wins).

## Session flow

1. Viewer enters peer ID → **signal** resolves + notifies agent.
2. Agent shows **consent** UI (or auto-accepts if unattended opt-in is enabled).
3. Candidate address exchange via signal.
4. **Hole punch** ≤ 3 s from the same UDP socket; else **relay**.
5. **Noise XX** handshake (E2E).
6. **Password challenge-response** inside encrypted channel.
7. **Capabilities** negotiation (codecs, max res, monitors, features).
8. Streaming starts with **IDR**; cursor + input paths active; visible host indicator on.

## Version negotiation

- Hello carries `app_version`; envelope `version` is `PROTOCOL_VERSION`.
- If peer version is unsupported → `SessionStop { reason: "protocol mismatch" }`.
- Breaking changes: bump `PROTOCOL_VERSION`, update this doc, add proptest roundtrip, note compat in CHANGELOG.

## Priority and head-of-line

- Input must **never** wait behind video. Use a dedicated high-priority reliable stream.
- Mouse-move is datagram + coalesce (latest position only).

## Security notes

- Password never appears on Control in cleartext before Noise finishes.
- Clipboard/file large payloads require consent flags.
- See [04_SECURITY.md](04_SECURITY.md).

## Related

Types: `crates/vd-proto` · Adaptation: [03_NETWORK_ADAPTATION.md](03_NETWORK_ADAPTATION.md)
