//! Channel payload types for the initial protocol (v1).

use serde::{Deserialize, Serialize};

/// Union of all channel payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Payload {
    /// Control channel messages.
    Control(ControlMessage),
    /// High-priority input (keys/buttons/wheel/text).
    Input(InputMessage),
    /// Unreliable mouse move.
    InputMove(InputMoveMessage),
    /// Video fragment.
    Video(VideoFrameFragment),
    /// Cursor shape update.
    Cursor(CursorMessage),
    /// Audio frame.
    Audio(AudioFrame),
    /// Receiver feedback.
    Feedback(FeedbackMessage),
    /// Clipboard sync.
    Clipboard(ClipboardMessage),
    /// File transfer.
    File(FileMessage),
    /// Chat text.
    Chat(ChatMessage),
}

/// Control-plane messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlMessage {
    /// Initial hello from either side.
    Hello {
        /// Soft name string (agent/viewer).
        role: String,
        /// Build / app version string.
        app_version: String,
    },
    /// Capabilities advertisement after auth.
    Capabilities {
        /// Supported codec ids (e.g. "h264", "h265").
        codecs: Vec<String>,
        /// Max encode width.
        max_width: u32,
        /// Max encode height.
        max_height: u32,
        /// Feature flags.
        features: Vec<String>,
    },
    /// Session lifecycle.
    SessionStart {
        /// Session id as bytes (u128 LE).
        session_id: [u8; 16],
    },
    /// End session (either side or consent revoke).
    SessionStop {
        /// Human-readable reason.
        reason: String,
    },
    /// Quality / adaptation hint from viewer or agent.
    QualityHint {
        /// Target bitrate bps (0 = unset).
        target_bps: u32,
        /// Preferred max fps.
        max_fps: u8,
        /// Scale percent (100 = native).
        scale_percent: u8,
    },
    /// Monitor enumeration.
    MonitorList {
        /// Monitors available on host.
        monitors: Vec<MonitorInfo>,
    },
    /// Round-trip ping.
    Ping {
        /// Echo nonce.
        nonce: u64,
    },
    /// Ping reply.
    Pong {
        /// Echo nonce.
        nonce: u64,
    },
    /// Password challenge (inside encrypted channel).
    AuthChallenge {
        /// Random challenge bytes.
        challenge: Vec<u8>,
    },
    /// Password challenge response.
    AuthResponse {
        /// Response proof bytes.
        response: Vec<u8>,
    },
    /// Auth result.
    AuthResult {
        /// Whether auth succeeded.
        ok: bool,
    },
}

/// Single monitor descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorInfo {
    /// Monitor index.
    pub index: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Display name.
    pub name: String,
}

/// Reliable input events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMessage {
    /// Key pressed.
    KeyDown {
        /// Platform-independent scancode / HID usage.
        scancode: u32,
        /// Modifier bitmask.
        modifiers: u32,
    },
    /// Key released.
    KeyUp {
        /// Platform-independent scancode / HID usage.
        scancode: u32,
        /// Modifier bitmask.
        modifiers: u32,
    },
    /// Mouse button.
    MouseButton {
        /// Button id (1=left, 2=right, 3=middle, …).
        button: u8,
        /// True = down, false = up.
        down: bool,
    },
    /// Scroll wheel.
    Wheel {
        /// Horizontal delta.
        dx: i32,
        /// Vertical delta.
        dy: i32,
    },
    /// Unicode / text injection.
    Text {
        /// UTF-8 text.
        text: String,
    },
    /// Request Ctrl+Alt+Del (service path on Windows).
    CadRequest,
}

/// Unreliable mouse position (coalesced, latest-wins).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputMoveMessage {
    /// Absolute X in desktop pixels (or normalized later via ADR).
    pub x: i32,
    /// Absolute Y in desktop pixels.
    pub y: i32,
    /// Monitor index.
    pub monitor: u32,
    /// True if relative delta instead of absolute.
    pub relative: bool,
}

/// Dirty rectangle within a frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    /// X origin.
    pub x: u32,
    /// Y origin.
    pub y: u32,
    /// Width.
    pub w: u32,
    /// Height.
    pub h: u32,
}

/// One fragment of an encoded video frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoFrameFragment {
    /// Frame identifier (monotonic per session).
    pub frame_id: u64,
    /// Fragment index (0-based).
    pub frag_idx: u16,
    /// Total fragments in this frame.
    pub frag_total: u16,
    /// Whether this frame is a keyframe / IDR.
    pub is_keyframe: bool,
    /// Capture timestamp (host monotonic µs).
    pub capture_ts_us: u64,
    /// Codec id string (e.g. "h264").
    pub codec: String,
    /// Encoded frame width.
    pub width: u16,
    /// Encoded frame height.
    pub height: u16,
    /// Optional dirty rects for this frame.
    pub dirty_rects: Option<Vec<Rect>>,
    /// Fragment payload bytes.
    pub data: Vec<u8>,
}

/// Cursor channel messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorMessage {
    /// Cacheable cursor shape.
    Shape {
        /// Cache id.
        id: u32,
        /// Hotspot X.
        hotspot_x: u16,
        /// Hotspot Y.
        hotspot_y: u16,
        /// Width.
        width: u16,
        /// Height.
        height: u16,
        /// RGBA pixels (or empty if system cursor id used).
        rgba: Vec<u8>,
    },
    /// Select previously cached shape.
    Select {
        /// Cache id.
        id: u32,
    },
    /// Hide cursor on viewer (host draws it in framebuffer — rare).
    Hide,
}

/// Opus audio frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioFrame {
    /// Sequence number.
    pub seq: u32,
    /// Capture timestamp µs.
    pub ts_us: u64,
    /// Opus packet bytes.
    pub data: Vec<u8>,
}

/// Feedback from viewer to agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackMessage {
    /// Periodic receiver report (~20 Hz).
    ReceiverReport {
        /// Received bitrate estimate (bps).
        recv_bps: u32,
        /// Loss in parts-per-million.
        loss_ppm: u32,
        /// Jitter in microseconds.
        jitter_us: u32,
        /// One-way delay gradient (signed, scaled).
        owd_gradient: i32,
        /// Decode time for last frame (ms).
        decode_ms: u16,
        /// Last fully received frame id.
        last_frame_id: u64,
        /// Request an IDR ASAP.
        need_idr: bool,
    },
    /// NACK for missing fragments of a frame.
    Nack {
        /// Frame id.
        frame_id: u64,
        /// Missing fragment indices.
        missing_frags: Vec<u16>,
    },
}

/// Clipboard messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardMessage {
    /// Offer clipboard content (consent required for large payloads).
    Offer {
        /// MIME-ish type.
        mime: String,
        /// Byte size.
        size: u32,
        /// Host/viewer already consented for this offer.
        consent: bool,
    },
    /// Clipboard data chunk.
    Data {
        /// MIME type.
        mime: String,
        /// Payload.
        bytes: Vec<u8>,
    },
    /// Clear remote clipboard mirror.
    Clear,
}

/// File transfer messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileMessage {
    /// Offer a file.
    Offer {
        /// Transfer id.
        transfer_id: u64,
        /// File name (basename).
        name: String,
        /// Size in bytes.
        size: u64,
        /// BLAKE3 hash if known (empty if streaming).
        blake3: Vec<u8>,
    },
    /// Accept a transfer.
    Accept {
        /// Transfer id.
        transfer_id: u64,
        /// Resume offset.
        resume_from: u64,
    },
    /// Data chunk.
    Chunk {
        /// Transfer id.
        transfer_id: u64,
        /// Offset.
        offset: u64,
        /// Bytes.
        data: Vec<u8>,
    },
    /// Acknowledge contiguous progress.
    Ack {
        /// Transfer id.
        transfer_id: u64,
        /// Contiguous bytes received.
        acked_through: u64,
    },
    /// Cancel.
    Cancel {
        /// Transfer id.
        transfer_id: u64,
        /// Reason.
        reason: String,
    },
    /// Complete + hash verify.
    Complete {
        /// Transfer id.
        transfer_id: u64,
        /// Final BLAKE3.
        blake3: Vec<u8>,
    },
}

/// Chat text message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// UTF-8 text (size-limited by application).
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::Channel;
    use crate::envelope::Envelope;
    use proptest::prelude::*;

    fn arb_rect() -> impl Strategy<Value = Rect> {
        (0u32..4096, 0u32..4096, 1u32..512, 1u32..512).prop_map(|(x, y, w, h)| Rect { x, y, w, h })
    }

    fn arb_video_frag() -> impl Strategy<Value = VideoFrameFragment> {
        (
            any::<u64>(),
            0u16..64,
            1u16..64,
            any::<bool>(),
            any::<u64>(),
            proptest::collection::vec(any::<u8>(), 0..64),
            16u16..1920,
            16u16..1080,
            proptest::option::of(proptest::collection::vec(arb_rect(), 0..4)),
        )
            .prop_map(
                |(
                    frame_id,
                    frag_idx,
                    frag_total,
                    is_keyframe,
                    capture_ts_us,
                    data,
                    width,
                    height,
                    dirty_rects,
                )| {
                    VideoFrameFragment {
                        frame_id,
                        frag_idx: frag_idx.min(frag_total.saturating_sub(1)),
                        frag_total: frag_total.max(1),
                        is_keyframe,
                        capture_ts_us,
                        codec: "h264".into(),
                        width,
                        height,
                        dirty_rects,
                        data,
                    }
                },
            )
    }

    fn arb_control() -> impl Strategy<Value = ControlMessage> {
        prop_oneof![
            Just(ControlMessage::Ping { nonce: 42 }),
            Just(ControlMessage::Pong { nonce: 42 }),
            Just(ControlMessage::SessionStop {
                reason: "test".into()
            }),
            Just(ControlMessage::AuthResult { ok: true }),
        ]
    }

    fn arb_payload() -> impl Strategy<Value = Payload> {
        prop_oneof![
            arb_control().prop_map(Payload::Control),
            arb_video_frag().prop_map(Payload::Video),
            Just(Payload::InputMove(InputMoveMessage {
                x: 10,
                y: 20,
                monitor: 0,
                relative: false,
            })),
            Just(Payload::Chat(ChatMessage {
                text: "hello".into()
            })),
            Just(Payload::Feedback(FeedbackMessage::ReceiverReport {
                recv_bps: 1_000_000,
                loss_ppm: 1000,
                jitter_us: 500,
                owd_gradient: -2,
                decode_ms: 4,
                last_frame_id: 7,
                need_idr: false,
            })),
        ]
    }

    proptest! {
        #[test]
        fn envelope_roundtrip(seq in any::<u64>(), ts in any::<u64>(), payload in arb_payload()) {
            let channel = match &payload {
                Payload::Control(_) => Channel::Control,
                Payload::Input(_) => Channel::Input,
                Payload::InputMove(_) => Channel::InputMove,
                Payload::Video(_) => Channel::Video,
                Payload::Cursor(_) => Channel::Cursor,
                Payload::Audio(_) => Channel::Audio,
                Payload::Feedback(_) => Channel::Feedback,
                Payload::Clipboard(_) => Channel::Clipboard,
                Payload::File(_) => Channel::File,
                Payload::Chat(_) => Channel::Chat,
            };
            let env = Envelope::new(channel, seq, ts, payload);
            let bytes = env.encode().expect("encode");
            let decoded = Envelope::decode(&bytes).expect("decode");
            prop_assert_eq!(env, decoded);
        }
    }

    #[test]
    fn hello_roundtrip_smoke() {
        let env = Envelope::new(
            Channel::Control,
            1,
            1000,
            Payload::Control(ControlMessage::Hello {
                role: "viewer".into(),
                app_version: "0.1.0".into(),
            }),
        );
        let bytes = env.encode().unwrap();
        let decoded = Envelope::decode(&bytes).unwrap();
        assert_eq!(env, decoded);
    }
}
