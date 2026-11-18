//! Deterministic patterned frame source for the P1 loopback pipeline.
//!
//! This is **not** a screen capturer: it synthesises a known BGRA pattern so the
//! capture → encode → decode → present path can be built and timed before DXGI
//! (T-0107) exists. It is deliberately available in normal builds — `tools/loopback`
//! and `vd-bench` use it — not just under `cfg(test)`.
//!
//! Pacing follows [`Capturer`] semantics: `capture_frame` returns `Ok(None)` when the
//! next frame is not due yet, which exercises the dirty-skip path of the pipeline.

use crate::{CapturedFrame, Capturer, PixelFormat};
use vd_common::{Error, MonotonicClock, Result};

/// Configuration for [`FakeCapturer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FakeCaptureConfig {
    /// Frame width in pixels (must be non-zero).
    pub width: u32,
    /// Frame height in pixels (must be non-zero).
    pub height: u32,
    /// Requested frames per second (must be non-zero).
    pub fps: u8,
}

impl Default for FakeCaptureConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 360,
            fps: 30,
        }
    }
}

impl FakeCaptureConfig {
    /// Microseconds between frames at the configured fps.
    #[must_use]
    pub fn frame_interval_us(&self) -> u64 {
        1_000_000 / u64::from(self.fps.max(1))
    }

    fn validate(&self) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Err(Error::InvalidArgument(
                "fake capture: width and height must be non-zero".into(),
            ));
        }
        if self.fps == 0 {
            return Err(Error::InvalidArgument(
                "fake capture: fps must be non-zero".into(),
            ));
        }
        if self.width > 16_384 || self.height > 16_384 {
            return Err(Error::InvalidArgument(
                "fake capture: dimensions out of range".into(),
            ));
        }
        Ok(())
    }
}

/// Render the deterministic BGRA test pattern for `frame_index`.
///
/// The pattern is: a blue→black horizontal ramp, a green vertical ramp, a bright
/// moving vertical bar (position derived from `frame_index`), and the low byte of
/// `frame_index` encoded in the alpha channel of the top row. Encoding the index
/// lets later stages prove frame identity end-to-end without a side channel.
#[must_use]
pub fn render_test_pattern(cfg: &FakeCaptureConfig, frame_index: u64) -> Vec<u8> {
    let width = cfg.width as usize;
    let height = cfg.height as usize;
    let mut data = vec![0u8; width * height * 4];

    let bar_x = if width == 0 {
        0
    } else {
        usize::try_from(frame_index % width as u64).unwrap_or(0)
    };
    let bar_width = (width / 16).max(4);
    let id_byte = (frame_index & 0xFF) as u8;

    for y in 0..height {
        let g = if height > 1 {
            (y * 255 / (height - 1)) as u8
        } else {
            0
        };
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let in_bar = x >= bar_x && x < bar_x.saturating_add(bar_width);
            data[idx] = if x < bar_x {
                (x * 255 / width.max(1)) as u8
            } else {
                (x * 255 / width.max(1) / 2) as u8
            }; // B
            data[idx + 1] = g; // G
            data[idx + 2] = if in_bar { 255 } else { 32 }; // R
            data[idx + 3] = if y == 0 { id_byte } else { 255 }; // A
        }
    }
    data
}

/// Synthesised patterned source implementing [`Capturer`].
#[derive(Debug)]
pub struct FakeCapturer {
    cfg: FakeCaptureConfig,
    clock: MonotonicClock,
    /// Test-only override; `None` means use the monotonic clock.
    manual_now_us: Option<u64>,
    next_due_us: u64,
    frame_index: u64,
    started: bool,
}

impl FakeCapturer {
    /// Create a fake capturer.
    ///
    /// # Errors
    /// Returns [`Error::InvalidArgument`] for zero-sized frames or `fps == 0`.
    pub fn new(cfg: FakeCaptureConfig) -> Result<Self> {
        cfg.validate()?;
        Ok(Self {
            cfg,
            clock: MonotonicClock::new(),
            manual_now_us: None,
            next_due_us: 0,
            frame_index: 0,
            started: false,
        })
    }

    /// Configuration this source was built with.
    #[must_use]
    pub fn config(&self) -> FakeCaptureConfig {
        self.cfg
    }

    /// Number of frames produced so far.
    #[must_use]
    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    /// Drive the clock manually instead of using wall time.
    ///
    /// Present so pacing and dirty-skip behaviour can be unit-tested without sleeps.
    #[cfg(test)]
    fn with_manual_clock(mut self) -> Self {
        self.manual_now_us = Some(0);
        self
    }

    /// Advance the manual clock. Test-only.
    #[cfg(test)]
    fn set_manual_now_us(&mut self, now_us: u64) {
        self.manual_now_us = Some(now_us);
    }

    fn now_us(&self) -> u64 {
        self.manual_now_us.unwrap_or_else(|| self.clock.now_us().0)
    }
}

impl Capturer for FakeCapturer {
    fn start(&mut self, _monitor_index: u32) -> Result<()> {
        self.started = true;
        self.frame_index = 0;
        self.next_due_us = self.now_us();
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<Option<CapturedFrame>> {
        if !self.started {
            return Err(Error::InvalidArgument(
                "fake capture: start() must be called first".into(),
            ));
        }
        let now_us = self.now_us();
        if now_us < self.next_due_us {
            return Ok(None);
        }
        self.next_due_us = now_us.saturating_add(self.cfg.frame_interval_us());

        let data = render_test_pattern(&self.cfg, self.frame_index);
        let frame = CapturedFrame {
            width: self.cfg.width,
            height: self.cfg.height,
            format: PixelFormat::Bgra8,
            capture_ts_us: now_us,
            dirty_rects: Vec::new(),
            data,
        };
        self.frame_index += 1;
        Ok(Some(frame))
    }

    fn stop(&mut self) -> Result<()> {
        self.started = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> FakeCaptureConfig {
        FakeCaptureConfig {
            width: 32,
            height: 16,
            fps: 30,
        }
    }

    #[test]
    fn rejects_degenerate_config() {
        assert!(FakeCapturer::new(FakeCaptureConfig { width: 0, ..cfg() }).is_err());
        assert!(FakeCapturer::new(FakeCaptureConfig { fps: 0, ..cfg() }).is_err());
    }

    #[test]
    fn interval_matches_fps() {
        assert_eq!(cfg().frame_interval_us(), 33_333);
        assert_eq!(
            FakeCaptureConfig { fps: 200, ..cfg() }.frame_interval_us(),
            5_000
        );
    }

    #[test]
    fn pattern_has_expected_layout() {
        let c = cfg();
        let data = render_test_pattern(&c, 0);
        assert_eq!(data.len(), (c.width * c.height * 4) as usize);
        // Bottom row alpha is opaque; top row alpha carries the frame index.
        let top_alpha = data[3];
        assert_eq!(top_alpha, 0);
        let bottom_alpha = data[data.len() - 1];
        assert_eq!(bottom_alpha, 255);
        // Green ramp increases down the image.
        let top_g = data[1];
        let mid_g = data[((c.height as usize / 2) * c.width as usize) * 4 + 1];
        assert!(mid_g > top_g);
    }

    #[test]
    fn pattern_encodes_frame_index_and_moves() {
        let c = cfg();
        let f0 = render_test_pattern(&c, 0);
        let f1 = render_test_pattern(&c, 1);
        assert_ne!(f0, f1, "consecutive frames must differ (motion bar)");
        assert_eq!(f0[3], 0);
        assert_eq!(f1[3], 1);
        assert_eq!(render_test_pattern(&c, 256)[3], 0, "index byte wraps");
    }

    #[test]
    fn yields_frames_at_configured_rate() {
        let mut cap = FakeCapturer::new(cfg()).unwrap().with_manual_clock();
        assert!(
            cap.capture_frame().is_err(),
            "capture before start must fail"
        );
        cap.start(0).unwrap();

        // First frame is due immediately.
        let f0 = cap.capture_frame().unwrap().expect("first frame");
        assert_eq!((f0.width, f0.height), (32, 16));
        assert_eq!(f0.format, PixelFormat::Bgra8);
        assert_eq!(f0.capture_ts_us, 0);
        assert_eq!(f0.data.len(), 32 * 16 * 4);

        // Too early for the next one: dirty-skip, not an error.
        assert!(cap.capture_frame().unwrap().is_none());

        // Just before the interval still yields nothing.
        cap.set_manual_now_us(cfg().frame_interval_us() - 1);
        assert!(cap.capture_frame().unwrap().is_none());

        // At the interval the next frame is due and timestamps advance.
        cap.set_manual_now_us(cfg().frame_interval_us());
        let f1 = cap.capture_frame().unwrap().expect("second frame");
        assert!(f1.capture_ts_us > f0.capture_ts_us);
        assert_eq!(cap.frame_index(), 2);
        assert_ne!(f0.data, f1.data, "pattern must animate");
    }
}
