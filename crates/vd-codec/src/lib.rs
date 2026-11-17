//! Video encode/decode abstraction. HW backends are stubs; openh264 comes in P1.
//!
//! Unsafe allowed for FFI; each `unsafe` needs `// SAFETY:`.

use vd_common::{Error, Result};

/// Codec identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecKind {
    /// H.264 / AVC.
    H264,
    /// H.265 / HEVC (later).
    H265,
    /// AV1 (later).
    Av1,
}

/// Encoder configuration (low-latency defaults).
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// Codec.
    pub codec: CodecKind,
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
    /// Target fps.
    pub fps: u8,
    /// Target bitrate bps.
    pub bitrate_bps: u32,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            codec: CodecKind::H264,
            width: 1920,
            height: 1080,
            fps: 30,
            bitrate_bps: 4_000_000,
        }
    }
}

/// Encoded bitstream unit (one access unit / frame).
#[derive(Debug, Clone)]
pub struct EncodedFrame {
    /// Bitstream bytes.
    pub data: Vec<u8>,
    /// Keyframe / IDR.
    pub is_keyframe: bool,
    /// Capture timestamp µs.
    pub capture_ts_us: u64,
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
}

/// Video encoder.
pub trait Encoder: Send {
    /// (Re)configure the encoder.
    ///
    /// # Errors
    /// Backend failures.
    fn configure(&mut self, cfg: &EncoderConfig) -> Result<()>;

    /// Encode one frame. `force_idr` requests a keyframe.
    ///
    /// # Errors
    /// Backend failures.
    fn encode(
        &mut self,
        frame_bgra: &[u8],
        capture_ts_us: u64,
        force_idr: bool,
    ) -> Result<EncodedFrame>;

    /// Update bitrate without full reconfigure when possible.
    ///
    /// # Errors
    /// Backend failures.
    fn set_bitrate(&mut self, bitrate_bps: u32) -> Result<()>;
}

/// Decoded raw frame.
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
    /// BGRA (stub path) or GPU handle later.
    pub data: Vec<u8>,
}

/// Video decoder.
pub trait Decoder: Send {
    /// Decode one access unit.
    ///
    /// # Errors
    /// Backend failures.
    fn decode(&mut self, data: &[u8], is_keyframe: bool) -> Result<DecodedFrame>;
}

/// Software H.264 encoder stub (openh264 wired in P1).
#[derive(Debug, Default)]
pub struct OpenH264Encoder {
    cfg: EncoderConfig,
}

impl Encoder for OpenH264Encoder {
    fn configure(&mut self, cfg: &EncoderConfig) -> Result<()> {
        self.cfg = cfg.clone();
        Err(Error::NotImplemented("openh264 Encoder::configure"))
    }

    fn encode(
        &mut self,
        _frame_bgra: &[u8],
        _capture_ts_us: u64,
        _force_idr: bool,
    ) -> Result<EncodedFrame> {
        Err(Error::NotImplemented("openh264 Encoder::encode"))
    }

    fn set_bitrate(&mut self, bitrate_bps: u32) -> Result<()> {
        self.cfg.bitrate_bps = bitrate_bps;
        Err(Error::NotImplemented("openh264 Encoder::set_bitrate"))
    }
}

/// Software H.264 decoder stub.
#[derive(Debug, Default)]
pub struct OpenH264Decoder;

impl Decoder for OpenH264Decoder {
    fn decode(&mut self, _data: &[u8], _is_keyframe: bool) -> Result<DecodedFrame> {
        Err(Error::NotImplemented("openh264 Decoder::decode"))
    }
}

/// Hardware encoder module stubs (NVENC / AMF / QSV).
pub mod hw {
    use super::{EncodedFrame, Encoder, EncoderConfig};
    use vd_common::{Error, Result};

    /// Placeholder HW encoder.
    #[derive(Debug, Default)]
    pub struct HwEncoder;

    impl Encoder for HwEncoder {
        fn configure(&mut self, _cfg: &EncoderConfig) -> Result<()> {
            Err(Error::NotImplemented("HW Encoder (NVENC/AMF/QSV)"))
        }

        fn encode(
            &mut self,
            _frame_bgra: &[u8],
            _capture_ts_us: u64,
            _force_idr: bool,
        ) -> Result<EncodedFrame> {
            Err(Error::NotImplemented("HW Encoder::encode"))
        }

        fn set_bitrate(&mut self, _bitrate_bps: u32) -> Result<()> {
            Err(Error::NotImplemented("HW Encoder::set_bitrate"))
        }
    }
}
