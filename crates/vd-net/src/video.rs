//! Video packetisation: split one encoded frame into MTU-safe fragments (T-0205).
//!
//! The wire type is [`vd_proto::VideoFrameFragment`]. Reassembly is
//! [`crate::reassemble`] (T-0206). Fragment payloads are capped at
//! [`DEFAULT_MAX_FRAGMENT_DATA`] (1150 B) so that the serialized envelope still fits the
//! 1200 B path MTU assumed by
//! [docs/03_NETWORK_ADAPTATION.md](../../../docs/03_NETWORK_ADAPTATION.md).
//!
//! # Why a borrowed view instead of `vd_codec::EncodedFrame`
//!
//! Feature crates only depend downward on `vd-proto`/`vd-common` (architecture §3), so
//! `vd-net` must not link `vd-codec` in its product graph. [`EncodedFrameRef`] mirrors
//! `vd_codec::EncodedFrame` field for field; callers (apps, tools, tests) copy those
//! fields across, which is cheap because the payload is borrowed.
//!
//! # Not here yet
//!
//! - Dirty rects: `vd_codec::EncodedFrame` does not carry them, so fragments are built
//!   with `dirty_rects: None` until capture metadata is threaded through the encoder.
//! - Pacing: fragments are produced immediately; the token-bucket pacer is T-0207.
//! - Retransmission/NACK/FEC: receiver-driven, see T-0404 / T-0406.

use vd_common::{Error, Result};
use vd_proto::VideoFrameFragment;

/// Path MTU baseline assumed by the protocol (bytes).
pub const MTU: usize = 1200;

/// Maximum fragment payload used by [`FragmentConfig::default`].
///
/// 1150 B leaves ~50 B for the envelope and QUIC/UDP/IP headers inside a 1200 B MTU.
pub const DEFAULT_MAX_FRAGMENT_DATA: usize = 1150;

/// Fragmentation knobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FragmentConfig {
    /// Maximum number of encoded bytes per [`VideoFrameFragment::data`].
    pub max_data_len: usize,
}

impl Default for FragmentConfig {
    fn default() -> Self {
        Self {
            max_data_len: DEFAULT_MAX_FRAGMENT_DATA,
        }
    }
}

impl FragmentConfig {
    /// Config with an explicit payload cap (used by tests and non-Ethernet paths).
    #[must_use]
    pub const fn with_max_data_len(max_data_len: usize) -> Self {
        Self { max_data_len }
    }
}

/// Borrowed view of one encoded frame.
///
/// Mirrors `vd_codec::EncodedFrame` without depending on the codec crate; see the module
/// docs. `width`/`height` are `u32` like the codec's, but must fit the wire's `u16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodedFrameRef<'a> {
    /// Monotonic frame identifier (per session).
    pub frame_id: u64,
    /// Whether this frame is a keyframe / IDR.
    pub is_keyframe: bool,
    /// Capture timestamp (host monotonic µs).
    pub capture_ts_us: u64,
    /// Codec id string, e.g. `"h264"`.
    pub codec: &'a str,
    /// Encoded width in pixels.
    pub width: u32,
    /// Encoded height in pixels.
    pub height: u32,
    /// Encoded bitstream bytes.
    pub data: &'a [u8],
}

/// Split one encoded frame into [`VideoFrameFragment`]s in send order.
///
/// Every fragment repeats the frame metadata and carries a contiguous slice of
/// `frame.data`; concatenating `data` in `frag_idx` order reproduces the input bitstream.
///
/// # Errors
/// Returns [`Error::InvalidArgument`] when the payload is empty, a dimension is zero or
/// exceeds `u16`, the codec id is empty, the fragment cap is zero, or the frame needs more
/// than `u16::MAX` fragments.
pub fn fragment_frame(
    frame: EncodedFrameRef<'_>,
    cfg: FragmentConfig,
) -> Result<Vec<VideoFrameFragment>> {
    if cfg.max_data_len == 0 {
        return Err(Error::InvalidArgument(
            "video: fragment max_data_len must be non-zero".into(),
        ));
    }
    if frame.data.is_empty() {
        return Err(Error::InvalidArgument(
            "video: encoded frame has no data".into(),
        ));
    }
    if frame.width == 0 || frame.height == 0 {
        return Err(Error::InvalidArgument(format!(
            "video: encoded frame has empty dimensions {}x{}",
            frame.width, frame.height
        )));
    }
    if frame.codec.is_empty() {
        return Err(Error::InvalidArgument(
            "video: encoded frame has an empty codec id".into(),
        ));
    }

    let width = u16::try_from(frame.width).map_err(|_| {
        Error::InvalidArgument(format!(
            "video: width {} does not fit the wire field (u16)",
            frame.width
        ))
    })?;
    let height = u16::try_from(frame.height).map_err(|_| {
        Error::InvalidArgument(format!(
            "video: height {} does not fit the wire field (u16)",
            frame.height
        ))
    })?;

    let frag_total = frame.data.len().div_ceil(cfg.max_data_len);
    let frag_total = u16::try_from(frag_total).map_err(|_| {
        Error::InvalidArgument(format!(
            "video: {} bytes at {} B per fragment exceeds {} fragments",
            frame.data.len(),
            cfg.max_data_len,
            u16::MAX
        ))
    })?;

    let mut fragments = Vec::with_capacity(usize::from(frag_total));
    for (idx, chunk) in frame.data.chunks(cfg.max_data_len).enumerate() {
        // `frag_total` was checked to fit u16 and `chunks()` yields exactly that many.
        fragments.push(VideoFrameFragment {
            frame_id: frame.frame_id,
            frag_idx: idx as u16,
            frag_total,
            is_keyframe: frame.is_keyframe,
            capture_ts_us: frame.capture_ts_us,
            codec: frame.codec.to_owned(),
            width,
            height,
            dirty_rects: None,
            data: chunk.to_vec(),
        });
    }
    debug_assert_eq!(fragments.len(), usize::from(frag_total));
    Ok(fragments)
}

#[cfg(test)]
mod tests {
    use super::{fragment_frame, EncodedFrameRef, FragmentConfig, DEFAULT_MAX_FRAGMENT_DATA};
    use vd_common::Error;

    fn frame(data: &[u8]) -> EncodedFrameRef<'_> {
        EncodedFrameRef {
            frame_id: 7,
            is_keyframe: false,
            capture_ts_us: 1_000,
            codec: "h264",
            width: 1280,
            height: 720,
            data,
        }
    }

    fn reassemble(fragments: &[vd_proto::VideoFrameFragment]) -> Vec<u8> {
        let mut ordered: Vec<_> = fragments.iter().collect();
        ordered.sort_by_key(|f| f.frag_idx);
        ordered
            .into_iter()
            .flat_map(|f| f.data.iter().copied())
            .collect()
    }

    #[test]
    fn exact_multiple_of_the_cap_has_no_short_tail() {
        let data = vec![0xAB; DEFAULT_MAX_FRAGMENT_DATA * 2];
        let fragments = fragment_frame(frame(&data), FragmentConfig::default()).expect("fragment");

        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[0].frag_idx, 0);
        assert_eq!(fragments[1].frag_idx, 1);
        assert_eq!(fragments[0].frag_total, 2);
        assert_eq!(fragments[1].data.len(), DEFAULT_MAX_FRAGMENT_DATA);
        assert_eq!(reassemble(&fragments), data);
    }

    #[test]
    fn remainder_is_a_short_last_fragment() {
        let data = vec![0x5A; DEFAULT_MAX_FRAGMENT_DATA + 50];
        let fragments = fragment_frame(frame(&data), FragmentConfig::default()).expect("fragment");

        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[0].data.len(), DEFAULT_MAX_FRAGMENT_DATA);
        assert_eq!(fragments[1].data.len(), 50);
        assert_eq!(fragments[1].frag_total, 2);
        assert_eq!(reassemble(&fragments), data);
    }

    #[test]
    fn a_small_keyframe_is_one_fragment_with_copied_metadata() {
        let data = [1u8, 2, 3, 4];
        let mut input = frame(&data);
        input.is_keyframe = true;
        input.frame_id = 9;

        let fragments = fragment_frame(input, FragmentConfig::default()).expect("fragment");

        assert_eq!(
            fragments,
            vec![vd_proto::VideoFrameFragment {
                frame_id: 9,
                frag_idx: 0,
                frag_total: 1,
                is_keyframe: true,
                capture_ts_us: 1_000,
                codec: "h264".to_owned(),
                width: 1280,
                height: 720,
                dirty_rects: None,
                data: vec![1, 2, 3, 4],
            }]
        );
    }

    #[test]
    fn custom_cap_is_respected() {
        let data = [9u8; 10];
        let fragments =
            fragment_frame(frame(&data), FragmentConfig::with_max_data_len(4)).expect("fragment");

        assert_eq!(fragments.len(), 3);
        assert_eq!(
            fragments.iter().map(|f| f.data.len()).collect::<Vec<_>>(),
            vec![4, 4, 2]
        );
        assert!(fragments.iter().all(|f| f.frag_total == 3));
        assert_eq!(reassemble(&fragments), data);
    }

    #[test]
    fn empty_and_invalid_inputs_are_rejected() {
        let ok_data = [1u8, 2, 3];

        assert!(matches!(
            fragment_frame(frame(&[]), FragmentConfig::default()),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            fragment_frame(frame(&ok_data), FragmentConfig::with_max_data_len(0)),
            Err(Error::InvalidArgument(_))
        ));

        let mut zero_width = frame(&ok_data);
        zero_width.width = 0;
        assert!(matches!(
            fragment_frame(zero_width, FragmentConfig::default()),
            Err(Error::InvalidArgument(_))
        ));

        let mut huge_height = frame(&ok_data);
        huge_height.height = u32::from(u16::MAX) + 1;
        assert!(matches!(
            fragment_frame(huge_height, FragmentConfig::default()),
            Err(Error::InvalidArgument(_))
        ));

        let mut no_codec = frame(&ok_data);
        no_codec.codec = "";
        assert!(matches!(
            fragment_frame(no_codec, FragmentConfig::default()),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn a_frame_needing_more_than_u16_max_fragments_is_rejected() {
        // 1-byte fragments over 65_536 bytes: one fragment too many for `frag_total`.
        let data = vec![0u8; usize::from(u16::MAX) + 1];
        assert!(matches!(
            fragment_frame(frame(&data), FragmentConfig::with_max_data_len(1)),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn boundary_sizes_are_exact() {
        for len in [1, DEFAULT_MAX_FRAGMENT_DATA - 1, DEFAULT_MAX_FRAGMENT_DATA] {
            let data = vec![0x11; len];
            let fragments =
                fragment_frame(frame(&data), FragmentConfig::default()).expect("fragment");
            assert_eq!(fragments.len(), 1, "len {len} should fit one fragment");
            assert_eq!(fragments[0].frag_total, 1);
            assert_eq!(reassemble(&fragments), data);
        }
        let data = vec![0x22; DEFAULT_MAX_FRAGMENT_DATA + 1];
        let fragments = fragment_frame(frame(&data), FragmentConfig::default()).expect("fragment");
        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[1].data.len(), 1);
    }
}
