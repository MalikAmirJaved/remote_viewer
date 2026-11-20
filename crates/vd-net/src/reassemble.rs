//! Video frame reassembly with latest-frame-wins (T-0206).
//!
//! Feed [`VideoFrameFragment`]s as they arrive (any order). When every index for a
//! `frame_id` is present, [`FrameReassembler::push`] returns the concatenated
//! bitstream. Completing a frame **abandons** every incomplete older `frame_id`
//! still in flight — stale video must never block a newer complete frame.
//!
//! Fragments with `frame_id` ≤ the last completed id are ignored (late/stale).

use std::collections::BTreeMap;

use vd_common::{Error, Result};
use vd_proto::{Rect, VideoFrameFragment};

/// One fully reassembled encoded frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembledFrame {
    /// Frame identifier (monotonic per session).
    pub frame_id: u64,
    /// Whether this frame is a keyframe / IDR.
    pub is_keyframe: bool,
    /// Capture timestamp (host monotonic µs).
    pub capture_ts_us: u64,
    /// Codec id string (e.g. `"h264"`).
    pub codec: String,
    /// Encoded width.
    pub width: u16,
    /// Encoded height.
    pub height: u16,
    /// Optional dirty rects copied from the first fragment that carried them.
    pub dirty_rects: Option<Vec<Rect>>,
    /// Concatenated bitstream in `frag_idx` order.
    pub data: Vec<u8>,
}

/// Counters for diagnostics and tests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReassemblerStats {
    /// Frames that finished and were returned from [`FrameReassembler::push`].
    pub completed: u64,
    /// Incomplete frames dropped because a newer frame completed.
    pub abandoned: u64,
    /// Fragments ignored because their `frame_id` was already completed (or older).
    pub stale_fragments: u64,
    /// Duplicate `frag_idx` for a frame already holding that slot.
    pub duplicate_fragments: u64,
}

/// Latest-frame-wins video reassembler.
#[derive(Debug, Default)]
pub struct FrameReassembler {
    in_progress: BTreeMap<u64, PartialFrame>,
    /// Highest `frame_id` already emitted. Fragments at or below this are stale.
    last_completed: Option<u64>,
    stats: ReassemblerStats,
}

#[derive(Debug)]
struct PartialFrame {
    frag_total: u16,
    is_keyframe: bool,
    capture_ts_us: u64,
    codec: String,
    width: u16,
    height: u16,
    dirty_rects: Option<Vec<Rect>>,
    slots: Vec<Option<Vec<u8>>>,
    filled: u16,
}

impl FrameReassembler {
    /// Create an empty reassembler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Diagnostic counters.
    #[must_use]
    pub fn stats(&self) -> ReassemblerStats {
        self.stats
    }

    /// Number of frames currently waiting for more fragments.
    #[must_use]
    pub fn in_progress_count(&self) -> usize {
        self.in_progress.len()
    }

    /// Ingest one fragment. Returns [`Some`] when that fragment completed its frame.
    ///
    /// # Errors
    /// [`Error::InvalidArgument`] for zero `frag_total`, `frag_idx` out of range, empty
    /// payload, or metadata that disagrees with a partial frame already started.
    pub fn push(&mut self, frag: VideoFrameFragment) -> Result<Option<AssembledFrame>> {
        validate_fragment(&frag)?;

        if self
            .last_completed
            .is_some_and(|done| frag.frame_id <= done)
        {
            self.stats.stale_fragments += 1;
            return Ok(None);
        }

        let frame_id = frag.frame_id;
        if let Some(partial) = self.in_progress.get_mut(&frame_id) {
            partial.ingest(frag, &mut self.stats)?;
        } else {
            let partial = PartialFrame::start(frag)?;
            self.in_progress.insert(frame_id, partial);
        }

        let done = self
            .in_progress
            .get(&frame_id)
            .is_some_and(PartialFrame::is_complete);
        if !done {
            return Ok(None);
        }

        let partial = self
            .in_progress
            .remove(&frame_id)
            .expect("just confirmed present");
        let assembled = partial.finish(frame_id);

        let abandoned = self.in_progress.range(..frame_id).count();
        self.stats.abandoned += abandoned as u64;
        self.in_progress.retain(|id, _| *id > frame_id);

        self.last_completed = Some(frame_id);
        self.stats.completed += 1;
        Ok(Some(assembled))
    }
}

impl PartialFrame {
    fn start(frag: VideoFrameFragment) -> Result<Self> {
        let total = usize::from(frag.frag_total);
        let mut slots = vec![None; total];
        let idx = usize::from(frag.frag_idx);
        slots[idx] = Some(frag.data);
        Ok(Self {
            frag_total: frag.frag_total,
            is_keyframe: frag.is_keyframe,
            capture_ts_us: frag.capture_ts_us,
            codec: frag.codec,
            width: frag.width,
            height: frag.height,
            dirty_rects: frag.dirty_rects,
            slots,
            filled: 1,
        })
    }

    fn ingest(&mut self, frag: VideoFrameFragment, stats: &mut ReassemblerStats) -> Result<()> {
        if frag.frag_total != self.frag_total
            || frag.is_keyframe != self.is_keyframe
            || frag.codec != self.codec
            || frag.width != self.width
            || frag.height != self.height
        {
            return Err(Error::InvalidArgument(format!(
                "video reassembly: fragment metadata mismatch for frame {}",
                frag.frame_id
            )));
        }
        // Prefer the first dirty_rects we saw; ignore later disagreements silently.
        if self.dirty_rects.is_none() {
            self.dirty_rects = frag.dirty_rects;
        }
        let idx = usize::from(frag.frag_idx);
        if self.slots[idx].is_some() {
            stats.duplicate_fragments += 1;
            return Ok(());
        }
        self.slots[idx] = Some(frag.data);
        self.filled += 1;
        Ok(())
    }

    fn is_complete(&self) -> bool {
        self.filled == self.frag_total
    }

    fn finish(self, frame_id: u64) -> AssembledFrame {
        let mut data = Vec::new();
        for slot in self.slots {
            data.extend_from_slice(&slot.expect("complete frame has every slot"));
        }
        AssembledFrame {
            frame_id,
            is_keyframe: self.is_keyframe,
            capture_ts_us: self.capture_ts_us,
            codec: self.codec,
            width: self.width,
            height: self.height,
            dirty_rects: self.dirty_rects,
            data,
        }
    }
}

fn validate_fragment(frag: &VideoFrameFragment) -> Result<()> {
    if frag.frag_total == 0 {
        return Err(Error::InvalidArgument(
            "video reassembly: frag_total must be non-zero".into(),
        ));
    }
    if frag.frag_idx >= frag.frag_total {
        return Err(Error::InvalidArgument(format!(
            "video reassembly: frag_idx {} >= frag_total {}",
            frag.frag_idx, frag.frag_total
        )));
    }
    if frag.data.is_empty() {
        return Err(Error::InvalidArgument(
            "video reassembly: fragment payload is empty".into(),
        ));
    }
    if frag.width == 0 || frag.height == 0 {
        return Err(Error::InvalidArgument(format!(
            "video reassembly: empty dimensions {}x{}",
            frag.width, frag.height
        )));
    }
    if frag.codec.is_empty() {
        return Err(Error::InvalidArgument(
            "video reassembly: empty codec id".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::FrameReassembler;
    use vd_common::Error;
    use vd_proto::VideoFrameFragment;

    fn frag(frame_id: u64, idx: u16, total: u16, data: &[u8]) -> VideoFrameFragment {
        VideoFrameFragment {
            frame_id,
            frag_idx: idx,
            frag_total: total,
            is_keyframe: frame_id == 1,
            capture_ts_us: frame_id * 1000,
            codec: "h264".into(),
            width: 320,
            height: 240,
            dirty_rects: None,
            data: data.to_vec(),
        }
    }

    #[test]
    fn out_of_order_fragments_complete_a_frame() {
        let mut r = FrameReassembler::new();
        assert!(r.push(frag(1, 1, 2, b"BB")).unwrap().is_none());
        let done = r.push(frag(1, 0, 2, b"AA")).unwrap().expect("complete");
        assert_eq!(done.data, b"AABB");
        assert_eq!(done.frame_id, 1);
        assert!(done.is_keyframe);
        assert_eq!(r.stats().completed, 1);
        assert_eq!(r.in_progress_count(), 0);
    }

    #[test]
    fn completing_a_newer_frame_abandons_incomplete_older() {
        let mut r = FrameReassembler::new();
        assert!(r.push(frag(1, 0, 3, b"a0")).unwrap().is_none());
        assert!(r.push(frag(1, 1, 3, b"a1")).unwrap().is_none());
        assert_eq!(r.in_progress_count(), 1);

        let done = r.push(frag(2, 0, 1, b"whole")).unwrap().expect("frame 2");
        assert_eq!(done.frame_id, 2);
        assert_eq!(done.data, b"whole");
        assert_eq!(r.stats().abandoned, 1);
        assert_eq!(r.in_progress_count(), 0);

        // Late fragment for the abandoned frame is stale.
        assert!(r.push(frag(1, 2, 3, b"a2")).unwrap().is_none());
        assert_eq!(r.stats().stale_fragments, 1);
    }

    #[test]
    fn duplicate_fragment_is_ignored() {
        let mut r = FrameReassembler::new();
        assert!(r.push(frag(1, 0, 2, b"AA")).unwrap().is_none());
        assert!(r.push(frag(1, 0, 2, b"XX")).unwrap().is_none());
        assert_eq!(r.stats().duplicate_fragments, 1);
        let done = r.push(frag(1, 1, 2, b"BB")).unwrap().expect("done");
        assert_eq!(done.data, b"AABB");
    }

    #[test]
    fn invalid_fragments_are_rejected() {
        let mut r = FrameReassembler::new();
        assert!(matches!(
            r.push(frag(1, 0, 0, b"x")),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            r.push(frag(1, 2, 2, b"x")),
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            r.push(frag(1, 0, 1, b"")),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn metadata_mismatch_is_rejected() {
        let mut r = FrameReassembler::new();
        assert!(r.push(frag(1, 0, 2, b"AA")).unwrap().is_none());
        let mut bad = frag(1, 1, 2, b"BB");
        bad.width = 999;
        assert!(matches!(r.push(bad), Err(Error::InvalidArgument(_))));
    }
}
