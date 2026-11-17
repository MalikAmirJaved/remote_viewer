# Glossary — remote_viewer

| Term | Meaning |
|------|---------|
| **Agent** | Host-side app (`vd-agent`) that captures the desktop and accepts control |
| **Viewer** | Client app (`vd-viewer`) that displays the remote screen and sends input |
| **Glass-to-glass** | Time from remote screen change to pixel visible on viewer |
| **IDR** | Instantaneous Decoder Refresh (H.264 keyframe) |
| **Latest-frame-wins** | Drop stale video rather than queueing delay |
| **Noise XX** | Mutual authenticated key exchange pattern used for E2E |
| **TOFU** | Trust On First Use key pinning |
| **Hole punch** | Simultaneous UDP to create a direct NAT mapping |
| **Relay** | Server that forwards opaque encrypted packets when direct fails |
| **Signal** | Server for IDs, session broker, candidate exchange |
| **Pacer** | Token-bucket sender that smooths bursts |
| **OWD gradient** | One-way delay trend used by rate control |
| **CAD** | Ctrl+Alt+Del request (Windows secure attention) |
| **Unattended** | Opt-in access without interactive consent each time |
| **vd-*** | Internal crate prefix (codename); product name is remote_viewer |

## Related

[00_PROJECT_BRIEF.md](00_PROJECT_BRIEF.md)
