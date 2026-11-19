//! In-process loopback video source for the viewer window.
//!
//! Fake desktop → H.264 encode → H.264 decode runs on **one worker OS thread**; the UI
//! thread only drains the newest decoded RGBA frame. This keeps decode off the UI hot
//! path (role rule for `viewer-ui`) and gives T-0105 something real to present before
//! the network exists in P2.
//!
//! The frame channel has capacity 1 and the worker evicts the oldest queued frame when
//! the UI falls behind. Dropping frames is correct here: presenting stale video is
//! worse than presenting fewer frames (architecture §2, latest-frame-wins).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use vd_capture::{Capturer, FakeCaptureConfig, FakeCapturer};
use vd_codec::{
    CodecKind, DecodedFrame, Decoder, Encoder, EncoderConfig, OpenH264Decoder, OpenH264Encoder,
};
use vd_common::{Error, Result};

/// Knobs for the viewer's built-in loopback source.
#[derive(Debug, Clone, Copy)]
pub struct LoopbackConfig {
    /// Frame width; must be even (I420).
    pub width: u32,
    /// Frame height; must be even (I420).
    pub height: u32,
    /// Frames per second requested from the fake desktop.
    pub fps: u8,
    /// Encoder bitrate in bps.
    pub bitrate_bps: u32,
}

impl Default for LoopbackConfig {
    fn default() -> Self {
        Self {
            width: 960,
            height: 540,
            fps: 30,
            bitrate_bps: 2_500_000,
        }
    }
}

impl LoopbackConfig {
    /// Reject configurations the worker cannot honour before any thread starts.
    ///
    /// # Errors
    /// [`Error::InvalidArgument`] for zero/odd dimensions, `fps == 0` or zero bitrate.
    pub fn validate(&self) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Err(Error::InvalidArgument(
                "viewer loopback: width and height must be non-zero".into(),
            ));
        }
        if self.width % 2 != 0 || self.height % 2 != 0 {
            return Err(Error::InvalidArgument(
                "viewer loopback: width and height must be even for I420".into(),
            ));
        }
        if self.fps == 0 {
            return Err(Error::InvalidArgument(
                "viewer loopback: fps must be non-zero".into(),
            ));
        }
        if self.bitrate_bps == 0 {
            return Err(Error::InvalidArgument(
                "viewer loopback: bitrate must be non-zero".into(),
            ));
        }
        Ok(())
    }
}

/// Handle to the decoding worker plus the newest-frame channel.
#[derive(Debug)]
pub struct LoopbackSource {
    frames: Receiver<DecodedFrame>,
    stop: Arc<AtomicBool>,
    dropped: Arc<AtomicU64>,
    produced: Arc<AtomicU64>,
    last_error: Arc<Mutex<Option<String>>>,
    worker: Option<JoinHandle<()>>,
}

impl LoopbackSource {
    /// Spawn the capture→encode→decode worker.
    ///
    /// # Errors
    /// Invalid configuration or failure to spawn the worker thread.
    pub fn start(cfg: LoopbackConfig) -> Result<Self> {
        cfg.validate()?;
        let (tx, rx) = bounded::<DecodedFrame>(1);
        let stop = Arc::new(AtomicBool::new(false));
        let dropped = Arc::new(AtomicU64::new(0));
        let produced = Arc::new(AtomicU64::new(0));
        let last_error = Arc::new(Mutex::new(None));

        let worker = {
            let stop = Arc::clone(&stop);
            let dropped = Arc::clone(&dropped);
            let produced = Arc::clone(&produced);
            let last_error = Arc::clone(&last_error);
            // The UI side keeps `rx`; the worker gets a handle to evict the oldest
            // queued frame when it fills up (same channel, cloned receiver).
            let worker_rx = rx.clone();
            thread::Builder::new()
                .name("vd-viewer-loopback".into())
                .spawn(move || {
                    let result = run_worker(cfg, &tx, &worker_rx, &stop, &dropped, &produced);
                    if let Err(e) = result {
                        if let Ok(mut slot) = last_error.lock() {
                            *slot = Some(e.to_string());
                        }
                    }
                })
                .map_err(|e| Error::Other(format!("viewer loopback: spawn worker: {e}")))?
        };

        Ok(Self {
            frames: rx,
            stop,
            dropped,
            produced,
            last_error,
            worker: Some(worker),
        })
    }

    /// Take the newest decoded frame, discarding anything older that queued up.
    ///
    /// Frames skipped here count towards [`Self::dropped`]: from the viewer's point of
    /// view they were lost to latest-frame-wins, exactly like worker-side evictions.
    #[must_use]
    pub fn try_recv_latest(&self) -> Option<DecodedFrame> {
        let mut newest = self.frames.try_recv().ok()?;
        while let Ok(frame) = self.frames.try_recv() {
            newest = frame;
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
        Some(newest)
    }

    /// Frames produced by the worker since start.
    #[must_use]
    pub fn produced(&self) -> u64 {
        self.produced.load(Ordering::Relaxed)
    }

    /// Frames lost to latest-frame-wins, either evicted by the worker or skipped when
    /// the UI drained more than one.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// Last worker error, if the worker stopped because of a failure.
    #[must_use]
    pub fn last_error(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|slot| slot.clone())
    }

    /// Whether the worker thread has finished (cleanly or not).
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.worker.as_ref().is_none_or(JoinHandle::is_finished)
    }

    /// Ask the worker to stop and wait for it. Idempotent.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for LoopbackSource {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_worker(
    cfg: LoopbackConfig,
    tx: &Sender<DecodedFrame>,
    rx: &Receiver<DecodedFrame>,
    stop: &AtomicBool,
    dropped: &AtomicU64,
    produced: &AtomicU64,
) -> Result<()> {
    let mut capturer = FakeCapturer::new(FakeCaptureConfig {
        width: cfg.width,
        height: cfg.height,
        fps: cfg.fps,
    })?;
    capturer.start(0)?;

    let mut encoder = OpenH264Encoder::new();
    encoder.configure(&EncoderConfig {
        codec: CodecKind::H264,
        width: cfg.width,
        height: cfg.height,
        fps: cfg.fps,
        bitrate_bps: cfg.bitrate_bps,
    })?;
    let mut decoder = OpenH264Decoder::new();

    let mut index = 0u64;
    while !stop.load(Ordering::Acquire) {
        let Some(frame) = capturer.capture_frame()? else {
            // Not due yet. Production capture blocks; the fake source paces itself.
            thread::sleep(Duration::from_millis(1));
            continue;
        };

        // One IDR up front; the loopback has no loss, so no recovery keyframes needed.
        let encoded = encoder.encode(&frame.data, frame.capture_ts_us, index == 0)?;
        let decoded = match decoder.decode(&encoded.data, encoded.is_keyframe) {
            Ok(frame) => frame,
            // A valid access unit can yield no picture; it is not a failure.
            Err(Error::Unavailable(_)) => continue,
            Err(e) => return Err(e),
        };
        index += 1;

        match tx.try_send(decoded) {
            Ok(()) => {
                produced.fetch_add(1, Ordering::Relaxed);
            }
            Err(TrySendError::Full(decoded)) => {
                // Latest-frame-wins: evict the stale frame, then retry once.
                let _ = rx.try_recv();
                dropped.fetch_add(1, Ordering::Relaxed);
                if tx.try_send(decoded).is_ok() {
                    produced.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(TrySendError::Disconnected(_)) => break,
        }
    }

    capturer.stop()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn small() -> LoopbackConfig {
        LoopbackConfig {
            width: 64,
            height: 48,
            fps: 30,
            bitrate_bps: 500_000,
        }
    }

    #[test]
    fn rejects_invalid_config_before_spawning() {
        for cfg in [
            LoopbackConfig {
                width: 0,
                ..small()
            },
            LoopbackConfig {
                height: 0,
                ..small()
            },
            LoopbackConfig {
                width: 65,
                ..small()
            },
            LoopbackConfig {
                height: 49,
                ..small()
            },
            LoopbackConfig { fps: 0, ..small() },
            LoopbackConfig {
                bitrate_bps: 0,
                ..small()
            },
        ] {
            assert!(matches!(
                LoopbackSource::start(cfg),
                Err(Error::InvalidArgument(_))
            ));
        }
    }

    #[test]
    fn worker_decodes_rgba_frames() {
        let source = LoopbackSource::start(small()).expect("start");

        let deadline = Instant::now() + Duration::from_secs(10);
        let mut seen = 0u32;
        let mut frame = None;
        while Instant::now() < deadline && seen < 5 {
            match source.try_recv_latest() {
                Some(f) => {
                    seen += 1;
                    frame = Some(f);
                }
                None => thread::sleep(Duration::from_millis(5)),
            }
        }

        assert_eq!(seen, 5, "worker must keep delivering frames");
        let frame = frame.expect("a frame");
        assert_eq!((frame.width, frame.height), (64, 48));
        assert_eq!(frame.data.len(), 64 * 48 * 4);
        assert!(
            frame.data.chunks_exact(4).all(|p| p[3] == 255),
            "decoded alpha must be opaque"
        );
        assert!(source.produced() >= 5);
        assert!(source.last_error().is_none(), "worker must not fail");
    }

    #[test]
    fn stop_joins_the_worker_and_is_idempotent() {
        let mut source = LoopbackSource::start(small()).expect("start");
        // Let it produce at least one frame so the stop path is not trivially early.
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline && source.produced() == 0 {
            thread::sleep(Duration::from_millis(5));
        }

        let started = Instant::now();
        source.stop();
        source.stop();
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "stop must not hang the UI thread"
        );
        assert!(source.is_finished());
    }

    #[test]
    fn dropping_the_source_stops_the_worker() {
        let source = LoopbackSource::start(small()).expect("start");
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline && source.produced() == 0 {
            thread::sleep(Duration::from_millis(5));
        }
        drop(source); // must not hang even though the worker is mid-frame
    }
}
