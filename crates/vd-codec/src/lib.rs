//! Video encode/decode abstraction. HW backends are stubs; the openh264 software
//! encoder is wired (T-0102), the decoder lands in T-0103.
//!
//! Unsafe allowed for FFI; each `unsafe` needs `// SAFETY:`.

use openh264::encoder::{
    BitRate, Complexity, Encoder as H264Encoder, EncoderConfig as H264EncoderConfig, FrameRate,
    FrameType, IntraFramePeriod, RateControlMode, UsageType,
};
use openh264::formats::{BgraSliceU8, YUVBuffer, YUVSource};
use openh264::{OpenH264API, Timestamp};
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
    /// Packed RGBA8 in the SW path (T-0103); a GPU handle replaces this in the HW path (T-0502).
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

/// Software H.264 encoder backed by Cisco OpenH264 (see ADR-0007).
///
/// Low-latency settings (screen content, CBR bitrate mode, infinite GOP with
/// on-demand IDR, single thread, lowest complexity, no frame skipping):
///
/// - No B-frames are ever produced by this configuration (OpenH264 real-time path).
/// - A keyframe is emitted on the first frame, after [`Encoder::configure`], after
///   [`Encoder::set_bitrate`], and whenever `force_idr` is set.
/// - Output is Annex-B NAL units (as returned by OpenH264), one access unit per call.
///
/// Frames are converted BGRA → I420 per call into a reused buffer, so steady-state
/// encoding does not allocate a YUV plane per frame. The BGRA→YUV conversion itself is
/// a CPU copy: acceptable for P1, replaced by GPU/NV12 in P5.
pub struct OpenH264Encoder {
    cfg: EncoderConfig,
    inner: Option<H264Encoder>,
    yuv: Option<YUVBuffer>,
    frames_encoded: u64,
}

impl std::fmt::Debug for OpenH264Encoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenH264Encoder")
            .field("cfg", &self.cfg)
            .field("configured", &self.inner.is_some())
            .field("yuv_allocated", &self.yuv.is_some())
            .field("frames_encoded", &self.frames_encoded)
            .finish()
    }
}

impl Default for OpenH264Encoder {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenH264Encoder {
    /// Create an unconfigured encoder. Call [`Encoder::configure`] before encoding.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cfg: EncoderConfig::default(),
            inner: None,
            yuv: None,
            frames_encoded: 0,
        }
    }

    /// Number of frames encoded since the last (re)configuration.
    #[must_use]
    pub fn frames_encoded(&self) -> u64 {
        self.frames_encoded
    }

    fn validate(cfg: &EncoderConfig) -> Result<()> {
        if cfg.codec != CodecKind::H264 {
            return Err(Error::InvalidArgument(
                "openh264 encoder only supports H264".into(),
            ));
        }
        if cfg.width == 0 || cfg.height == 0 {
            return Err(Error::InvalidArgument(
                "encoder: width and height must be non-zero".into(),
            ));
        }
        // 4:2:0 requires even dimensions on both axes.
        if cfg.width % 2 != 0 || cfg.height % 2 != 0 {
            return Err(Error::InvalidArgument(
                "encoder: width and height must be even for I420".into(),
            ));
        }
        if cfg.fps == 0 {
            return Err(Error::InvalidArgument(
                "encoder: fps must be non-zero".into(),
            ));
        }
        if cfg.bitrate_bps == 0 {
            return Err(Error::InvalidArgument(
                "encoder: bitrate must be non-zero".into(),
            ));
        }
        Ok(())
    }

    fn h264_config(cfg: &EncoderConfig) -> H264EncoderConfig {
        H264EncoderConfig::new()
            .usage_type(UsageType::ScreenContentRealTime)
            .rate_control_mode(RateControlMode::Bitrate)
            .bitrate(BitRate::from_bps(cfg.bitrate_bps))
            .max_frame_rate(FrameRate::from_hz(f32::from(cfg.fps)))
            // One output per input keeps the P1 loopback 1:1 and timing meaningful.
            .skip_frames(false)
            // On-demand IDR only: no surprise periodic or scene-change keyframes.
            .intra_frame_period(IntraFramePeriod::from_num_frames(0))
            .scene_change_detect(false)
            // Desktop content: let the rate controller, not heuristics, decide detail.
            .adaptive_quantization(false)
            .background_detection(false)
            // Latency over compression efficiency for the SW path.
            .complexity(Complexity::Low)
            .num_threads(1)
    }

    fn rebuild(&mut self) -> Result<()> {
        let api = OpenH264API::from_source();
        let encoder = H264Encoder::with_api_config(api, Self::h264_config(&self.cfg))
            .map_err(|e| Error::Other(format!("openh264 init failed: {e}")))?;
        self.inner = Some(encoder);
        self.frames_encoded = 0;
        Ok(())
    }
}

impl Encoder for OpenH264Encoder {
    fn configure(&mut self, cfg: &EncoderConfig) -> Result<()> {
        Self::validate(cfg)?;
        self.cfg = cfg.clone();
        self.yuv = None;
        self.rebuild()
    }

    fn encode(
        &mut self,
        frame_bgra: &[u8],
        capture_ts_us: u64,
        force_idr: bool,
    ) -> Result<EncodedFrame> {
        let width = self.cfg.width as usize;
        let height = self.cfg.height as usize;
        let expected = width * height * 4;
        if frame_bgra.len() != expected {
            return Err(Error::InvalidArgument(format!(
                "encoder: expected {expected} BGRA bytes for {width}x{height}, got {}",
                frame_bgra.len()
            )));
        }

        {
            let yuv = self
                .yuv
                .get_or_insert_with(|| YUVBuffer::new(width, height));
            yuv.read_bgra8(BgraSliceU8::new(frame_bgra, (width, height)));
        }

        let inner = self.inner.as_mut().ok_or_else(|| {
            Error::InvalidArgument("encoder: configure() must be called before encode()".into())
        })?;
        if force_idr {
            inner.force_intra_frame();
        }

        let (data, is_keyframe) = {
            let stream = inner
                .encode_at(
                    self.yuv.as_ref().expect("yuv buffer was just filled"),
                    Timestamp::from_millis(capture_ts_us / 1_000),
                )
                .map_err(|e| Error::Other(format!("openh264 encode failed: {e}")))?;
            let is_keyframe = matches!(stream.frame_type(), FrameType::IDR | FrameType::I);
            (stream.to_vec(), is_keyframe)
        };

        self.frames_encoded += 1;
        Ok(EncodedFrame {
            data,
            is_keyframe,
            capture_ts_us,
            width: self.cfg.width,
            height: self.cfg.height,
        })
    }

    /// Rebuild the encoder at a new bitrate.
    ///
    /// OpenH264 binds the bitrate at initialization, so this re-creates the encoder
    /// (a fresh keyframe follows automatically). Treat it as a slow path driven by
    /// `RateDecision` (T-0403), not a per-frame call.
    fn set_bitrate(&mut self, bitrate_bps: u32) -> Result<()> {
        if bitrate_bps == 0 {
            return Err(Error::InvalidArgument(
                "encoder: bitrate must be non-zero".into(),
            ));
        }
        if self.cfg.bitrate_bps == bitrate_bps {
            return Ok(());
        }
        self.cfg.bitrate_bps = bitrate_bps;
        if self.inner.is_some() {
            self.rebuild()?;
        }
        Ok(())
    }
}

/// Software H.264 decoder backed by Cisco OpenH264 (see ADR-0007).
///
/// Input is Annex-B NAL units (what [`OpenH264Encoder`] emits). Output is packed
/// **RGBA8**, which is what the viewer UI wants for texture upload and what OpenH264
/// produces directly — no extra channel-swap pass. The decoder keeps reference frames
/// internally, so sequential P frames must be decoded in order.
pub struct OpenH264Decoder {
    inner: Option<openh264::decoder::Decoder>,
    decoded_frames: u64,
}

impl std::fmt::Debug for OpenH264Decoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenH264Decoder")
            .field("initialized", &self.inner.is_some())
            .field("decoded_frames", &self.decoded_frames)
            .finish()
    }
}

impl Default for OpenH264Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenH264Decoder {
    /// Create an uninitialized decoder; OpenH264 state is created on first decode.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: None,
            decoded_frames: 0,
        }
    }

    /// Number of frames produced since construction.
    #[must_use]
    pub fn decoded_frames(&self) -> u64 {
        self.decoded_frames
    }
}

impl Decoder for OpenH264Decoder {
    fn decode(&mut self, data: &[u8], _is_keyframe: bool) -> Result<DecodedFrame> {
        if data.is_empty() {
            return Err(Error::InvalidArgument("decoder: empty bitstream".into()));
        }

        if self.inner.is_none() {
            self.inner = Some(
                openh264::decoder::Decoder::new()
                    .map_err(|e| Error::Other(format!("openh264 decoder init failed: {e}")))?,
            );
        }
        let decoder = self.inner.as_mut().expect("decoder was just initialized");

        let yuv = decoder
            .decode(data)
            .map_err(|e| Error::Other(format!("openh264 decode failed: {e}")))?
            .ok_or_else(|| {
                Error::Unavailable("decoder produced no frame for this access unit".into())
            })?;

        let (width, height) = yuv.dimensions();
        if width == 0 || height == 0 {
            return Err(Error::Unavailable("decoder produced an empty frame".into()));
        }
        let mut rgba = vec![0u8; width * height * 4];
        yuv.write_rgba8(&mut rgba);

        self.decoded_frames += 1;
        Ok(DecodedFrame {
            width: width as u32,
            height: height as u32,
            data: rgba,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> EncoderConfig {
        EncoderConfig {
            codec: CodecKind::H264,
            width: 64,
            height: 48,
            fps: 30,
            bitrate_bps: 500_000,
        }
    }

    /// Moving vertical bar on a gradient: enough motion to produce real P frames,
    /// cheap enough to keep unit tests instant.
    fn frame(shift: u32) -> Vec<u8> {
        let (w, h) = (64u32, 48u32);
        let mut buf = vec![255u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx] = (x * 4) as u8;
                buf[idx + 1] = (y * 5) as u8;
                buf[idx + 2] = if (x + shift) % w < 8 { 255 } else { 16 };
                buf[idx + 3] = 255;
            }
        }
        buf
    }

    fn is_annex_b(data: &[u8]) -> bool {
        data.starts_with(&[0, 0, 0, 1]) || data.starts_with(&[0, 0, 1])
    }

    #[test]
    fn first_frame_is_annex_b_keyframe() {
        let mut enc = OpenH264Encoder::new();
        enc.configure(&cfg()).expect("configure");

        let out = enc.encode(&frame(0), 0, false).expect("encode");
        assert!(!out.data.is_empty(), "encoder produced no bytes");
        assert!(is_annex_b(&out.data), "expected Annex-B start code");
        assert!(
            out.data.len() < frame(0).len(),
            "encoded output ({}) should be smaller than raw BGRA ({})",
            out.data.len(),
            frame(0).len()
        );
        assert!(out.is_keyframe, "first frame must be IDR/I");
        assert_eq!((out.width, out.height), (64, 48));
        assert_eq!(out.capture_ts_us, 0);
        assert_eq!(enc.frames_encoded(), 1);
    }

    #[test]
    fn subsequent_frames_are_deltas_and_force_idr_overrides() {
        let mut enc = OpenH264Encoder::new();
        enc.configure(&cfg()).expect("configure");

        let first = enc.encode(&frame(0), 0, false).expect("first");
        assert!(first.is_keyframe);

        let second = enc.encode(&frame(1), 33_000, false).expect("second");
        assert!(
            !second.is_keyframe,
            "second frame must be a P frame when not forced"
        );
        assert_ne!(second.data.len(), 0, "delta frame must carry bytes");

        let forced = enc.encode(&frame(2), 66_000, true).expect("forced");
        assert!(forced.is_keyframe, "force_idr must emit a keyframe");
        assert_eq!(forced.capture_ts_us, 66_000);

        // The encoder keeps producing deltas after the forced IDR.
        let after = enc.encode(&frame(3), 99_000, false).expect("after");
        assert!(!after.is_keyframe);
        assert_eq!(enc.frames_encoded(), 4);
    }

    #[test]
    fn set_bitrate_rebuilds_and_next_frame_is_keyframe() {
        let mut enc = OpenH264Encoder::new();
        enc.configure(&cfg()).expect("configure");
        let _ = enc.encode(&frame(0), 0, false).expect("first");
        let _ = enc.encode(&frame(1), 33_000, false).expect("second");

        enc.set_bitrate(250_000).expect("set_bitrate");
        assert_eq!(enc.frames_encoded(), 0, "rebuild resets the counter");

        let out = enc.encode(&frame(2), 66_000, false).expect("after rebuild");
        assert!(
            out.is_keyframe,
            "a fresh encoder must start with a keyframe"
        );

        // Idempotent when the bitrate is unchanged.
        enc.set_bitrate(250_000).expect("no-op set_bitrate");
        assert_eq!(enc.frames_encoded(), 1);
    }

    #[test]
    fn rejects_invalid_config_and_frame_size() {
        let mut enc = OpenH264Encoder::new();

        // Encoding before configure.
        let err = enc.encode(&frame(0), 0, false).unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)));

        // Odd dimensions break 4:2:0.
        let err = enc
            .configure(&EncoderConfig { width: 63, ..cfg() })
            .unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)));

        // Only H.264 is wired.
        let err = enc
            .configure(&EncoderConfig {
                codec: CodecKind::H265,
                ..cfg()
            })
            .unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)));

        // Wrong frame buffer size.
        enc.configure(&cfg()).expect("configure");
        let err = enc.encode(&[0u8; 10], 0, false).unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)));
        assert_eq!(enc.frames_encoded(), 0, "failed encode must not count");
    }

    #[test]
    fn zero_bitrate_is_rejected() {
        let mut enc = OpenH264Encoder::new();
        enc.configure(&cfg()).expect("configure");
        let err = enc.set_bitrate(0).unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)));
    }

    /// Mean absolute RGB difference between a source BGRA frame and a decoded RGBA frame.
    fn mean_abs_diff(src_bgra: &[u8], decoded_rgba: &[u8]) -> f64 {
        assert_eq!(src_bgra.len(), decoded_rgba.len(), "size mismatch");
        let pixels = src_bgra.len() / 4;
        let mut sum = 0u64;
        for i in 0..pixels {
            let s = &src_bgra[i * 4..i * 4 + 3];
            let d = &decoded_rgba[i * 4..i * 4 + 3];
            // Source is BGRA, decoded is RGBA: compare R,G,B in the same order.
            sum += u64::from(s[2].abs_diff(d[0]));
            sum += u64::from(s[1].abs_diff(d[1]));
            sum += u64::from(s[0].abs_diff(d[2]));
        }
        sum as f64 / (pixels as f64 * 3.0)
    }

    #[test]
    fn keyframe_roundtrips_through_decoder() {
        let mut enc = OpenH264Encoder::new();
        enc.configure(&cfg()).expect("configure");
        let source = frame(0);
        let encoded = enc.encode(&source, 0, false).expect("encode");

        let mut dec = OpenH264Decoder::new();
        let decoded = dec
            .decode(&encoded.data, encoded.is_keyframe)
            .expect("decode");
        assert_eq!((decoded.width, decoded.height), (64, 48));
        assert_eq!(decoded.data.len(), 64 * 48 * 4);

        let diff = mean_abs_diff(&source, &decoded.data);
        println!("keyframe mean abs diff: {diff:.2}");
        assert!(
            diff < 12.0,
            "decoded keyframe deviates too far from source: {diff:.2}"
        );
        assert!(
            decoded.data.chunks_exact(4).all(|p| p[3] == 255),
            "decoded alpha must be opaque"
        );
        assert_eq!(dec.decoded_frames(), 1);
    }

    #[test]
    fn delta_sequence_roundtrips_in_order() {
        let mut enc = OpenH264Encoder::new();
        enc.configure(&cfg()).expect("configure");
        let mut dec = OpenH264Decoder::new();

        for i in 0..5u32 {
            let source = frame(i);
            let encoded = enc
                .encode(&source, u64::from(i) * 33_000, i == 0)
                .expect("encode");
            let decoded = dec
                .decode(&encoded.data, encoded.is_keyframe)
                .expect("decode");
            assert_eq!((decoded.width, decoded.height), (64, 48), "frame {i}");
            let diff = mean_abs_diff(&source, &decoded.data);
            assert!(diff < 12.0, "frame {i} diff {diff:.2}");
        }
        assert_eq!(dec.decoded_frames(), 5);
        assert_eq!(enc.frames_encoded(), 5);
    }

    #[test]
    fn decoder_rejects_empty_and_garbage_input() {
        let mut dec = OpenH264Decoder::new();
        let err = dec.decode(&[], true).unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)));

        // Neither a panic nor a silent success.
        let garbage = [0u8, 0, 0, 1, 0x09, 0x09, 0x09, 0x09];
        assert!(dec.decode(&garbage, true).is_err());
        assert_eq!(dec.decoded_frames(), 0);
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
