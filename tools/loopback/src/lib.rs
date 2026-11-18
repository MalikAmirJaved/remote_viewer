//! Single-process capture → encode → decode → present pipeline (P1 loopback).
//!
//! Public surface so `tools/vd-bench` can measure per-stage latency (T-0106) without
//! importing a product binary.
//!
//! Design: [docs/01_ARCHITECTURE.md](../../../docs/01_ARCHITECTURE.md) §2 —
//! bounded `crossbeam-channel` per stage, capacity 1–2, latest-frame-wins. Each stage
//! runs on its own OS thread and never blocks the producer: [`LatestSender::send_latest`]
//! evicts the oldest queued item instead of waiting, so capture keeps its cadence when
//! encode falls behind.
//!
//! Scope of this harness: the network/pacer stage is bypassed (encode feeds decode
//! directly), and "present" is a stand-in until the egui viewer lands in T-0105. The
//! timings it reports are therefore host-side pipeline costs, not glass-to-glass
//! latency.
#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use vd_capture::{CapturedFrame, Capturer, FakeCaptureConfig, FakeCapturer};
use vd_codec::{
    CodecKind, DecodedFrame, Decoder, EncodedFrame, Encoder, EncoderConfig, OpenH264Decoder,
    OpenH264Encoder,
};
use vd_common::{Error, Result};

/// Loopback pipeline knobs.
#[derive(Debug, Clone, Copy)]
pub struct PipelineConfig {
    /// Frames per second requested from the fake source.
    pub fps: u8,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Target encoder bitrate in bps.
    pub bitrate_bps: u32,
    /// Frames to push through before the pipeline shuts down.
    pub frames: u32,
    /// Per-stage queue capacity (1–2, per architecture §2).
    pub queue_cap: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            fps: 30,
            width: 1280,
            height: 720,
            bitrate_bps: 4_000_000,
            frames: 300,
            queue_cap: 2,
        }
    }
}

impl PipelineConfig {
    /// Reject configurations the pipeline cannot honour before any thread starts.
    fn validate(&self) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Err(Error::InvalidArgument(
                "loopback: width and height must be non-zero".into(),
            ));
        }
        if self.width % 2 != 0 || self.height % 2 != 0 {
            return Err(Error::InvalidArgument(
                "loopback: width and height must be even for I420".into(),
            ));
        }
        if self.fps == 0 {
            return Err(Error::InvalidArgument(
                "loopback: fps must be non-zero".into(),
            ));
        }
        if self.bitrate_bps == 0 {
            return Err(Error::InvalidArgument(
                "loopback: bitrate must be non-zero".into(),
            ));
        }
        if self.frames == 0 {
            return Err(Error::InvalidArgument(
                "loopback: frames must be non-zero".into(),
            ));
        }
        if !(1..=2).contains(&self.queue_cap) {
            return Err(Error::InvalidArgument(format!(
                "loopback: queue_cap must be 1 or 2, got {}",
                self.queue_cap
            )));
        }
        Ok(())
    }
}

/// Per-stage timings for one frame. All values are microseconds.
#[derive(Debug, Clone, Copy, Default)]
pub struct StageTimings {
    /// Capture stage duration.
    pub capture: u64,
    /// Encode stage duration.
    pub encode: u64,
    /// Decode stage duration.
    pub decode: u64,
    /// Present stage duration.
    pub present: u64,
}

impl StageTimings {
    /// Sum of all stage durations.
    #[must_use]
    pub fn total_us(&self) -> u64 {
        self.capture + self.encode + self.decode + self.present
    }

    fn accumulate(&mut self, other: Self) {
        self.capture += other.capture;
        self.encode += other.encode;
        self.decode += other.decode;
        self.present += other.present;
    }
}

/// Counters from one pipeline run.
#[derive(Debug, Clone, Copy, Default)]
pub struct RunStats {
    /// Frames produced by capture.
    pub frames_captured: u64,
    /// Frames encoded.
    pub frames_encoded: u64,
    /// Frames decoded.
    pub frames_decoded: u64,
    /// Frames presented.
    pub frames_presented: u64,
    /// Items evicted between capture and encode.
    pub dropped_before_encode: u64,
    /// Items evicted between encode and decode.
    pub dropped_before_decode: u64,
    /// Items evicted between decode and present.
    pub dropped_before_present: u64,
    /// Summed per-stage timings over presented frames.
    pub timings: StageTimings,
    /// Worst single-frame total across presented frames.
    pub max_frame_total_us: u64,
    /// Index of the last presented frame (proves ordering/identity).
    pub last_presented_index: Option<u64>,
}

impl RunStats {
    /// Total evictions across all queues.
    #[must_use]
    pub fn dropped_total(&self) -> u64 {
        self.dropped_before_encode + self.dropped_before_decode + self.dropped_before_present
    }

    /// Mean total per-frame pipeline time over presented frames.
    #[must_use]
    pub fn mean_frame_total_us(&self) -> u64 {
        self.timings
            .total_us()
            .checked_div(self.frames_presented)
            .unwrap_or(0)
    }
}

/// Bounded channel sender that evicts the oldest queued item instead of blocking.
///
/// This is the "latest-frame-wins" policy from architecture §2: a slow consumer
/// costs frames, never capture cadence.
#[derive(Debug, Clone)]
pub struct LatestSender<T> {
    tx: Sender<T>,
    /// Kept so the sender can pop the oldest item on a full queue.
    rx: Receiver<T>,
    /// Set by [`LatestReceiver`]'s `Drop`. Without it the sender's own receiver clone
    /// would keep the channel "connected" forever, so a dead consumer would spin the
    /// producer instead of ending it.
    closed: Arc<AtomicBool>,
}

impl<T> LatestSender<T> {
    /// Push `value`, dropping the oldest queued item if the queue is full.
    ///
    /// Returns `Ok(true)` when an older item was evicted. Returns `Err(value)` when
    /// the consumer is gone, handing the value back so the caller can stop cleanly.
    ///
    /// # Errors
    /// The value itself is returned if the consumer has been dropped.
    pub fn send_latest(&self, value: T) -> std::result::Result<bool, T> {
        if self.closed.load(Ordering::Acquire) {
            return Err(value);
        }
        match self.tx.try_send(value) {
            Ok(()) => Ok(false),
            Err(TrySendError::Full(value)) => {
                // Full: evict exactly one (the oldest) and retry. Because the queue was
                // full, that retry can only fail if the consumer disconnected.
                let _evicted = self.rx.try_recv();
                match self.tx.try_send(value) {
                    Ok(()) => Ok(true),
                    Err(TrySendError::Full(v) | TrySendError::Disconnected(v)) => Err(v),
                }
            }
            Err(TrySendError::Disconnected(value)) => Err(value),
        }
    }
}

/// Receiving end of a latest-frame channel. Dropping it tells the sender to stop.
#[derive(Debug)]
pub struct LatestReceiver<T> {
    rx: Receiver<T>,
    closed: Arc<AtomicBool>,
}

impl<T> Drop for LatestReceiver<T> {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::Release);
    }
}

impl<T> std::ops::Deref for LatestReceiver<T> {
    type Target = Receiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

/// Create a bounded latest-frame channel with the given capacity (must be ≥ 1).
#[must_use]
pub fn latest_channel<T>(cap: usize) -> (LatestSender<T>, LatestReceiver<T>) {
    assert!(cap >= 1, "latest_channel capacity must be at least 1");
    let (tx, rx) = bounded::<T>(cap);
    let closed = Arc::new(AtomicBool::new(false));
    (
        LatestSender {
            tx,
            rx: rx.clone(),
            closed: Arc::clone(&closed),
        },
        LatestReceiver { rx, closed },
    )
}

/// A captured frame plus the cost of capturing it.
struct RawItem {
    index: u64,
    frame: CapturedFrame,
    capture_us: u64,
}

/// An encoded frame plus upstream stage costs.
struct EncodedItem {
    index: u64,
    encoded: EncodedFrame,
    capture_us: u64,
    encode_us: u64,
}

/// A decoded frame plus upstream stage costs.
struct DecodedItem {
    index: u64,
    decoded: DecodedFrame,
    capture_us: u64,
    encode_us: u64,
    decode_us: u64,
}

/// What the present stage reports back.
#[derive(Debug, Default)]
struct PresentOutcome {
    presented: u64,
    last_index: Option<u64>,
    timings: StageTimings,
    max_frame_total_us: u64,
}

fn dur_us(d: Duration) -> u64 {
    u64::try_from(d.as_micros()).unwrap_or(u64::MAX)
}

fn thread_error(stage: &str, err: &(dyn std::any::Any + Send)) -> Error {
    let detail = err
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| err.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".to_owned());
    Error::Other(format!("loopback {stage} thread panicked: {detail}"))
}

/// Run the loopback pipeline to completion and report per-stage costs.
///
/// # Errors
/// Invalid configuration, backend failure in any stage, or a stage thread panic.
pub fn run(cfg: PipelineConfig) -> Result<RunStats> {
    cfg.validate()?;

    let (cap_tx, enc_rx) = latest_channel::<RawItem>(cfg.queue_cap);
    let (enc_tx, dec_rx) = latest_channel::<EncodedItem>(cfg.queue_cap);
    let (dec_tx, out_rx) = latest_channel::<DecodedItem>(cfg.queue_cap);

    let capture = thread::Builder::new()
        .name("loopback-capture".into())
        .spawn(move || -> Result<(u64, u64)> {
            let mut capturer = FakeCapturer::new(FakeCaptureConfig {
                width: cfg.width,
                height: cfg.height,
                fps: cfg.fps,
            })?;
            capturer.start(0)?;

            let mut index = 0u64;
            let mut dropped = 0u64;
            while index < u64::from(cfg.frames) {
                let started = Instant::now();
                let Some(frame) = capturer.capture_frame()? else {
                    // Not due yet: poll at 1 ms. Production capture blocks in DXGI;
                    // this is a harness-only pacing detail, not a hot path to copy.
                    thread::sleep(Duration::from_millis(1));
                    continue;
                };
                let capture_us = dur_us(started.elapsed());
                let item = RawItem {
                    index,
                    frame,
                    capture_us,
                };
                index += 1;
                match cap_tx.send_latest(item) {
                    Ok(true) => dropped += 1,
                    Ok(false) => {}
                    Err(_) => break, // encode stage is gone; nothing left to do
                }
            }
            capturer.stop()?;
            Ok((index, dropped))
        })
        .map_err(|e| Error::Other(format!("loopback: spawn capture: {e}")))?;

    let (width, height, fps, bitrate_bps) = (cfg.width, cfg.height, cfg.fps, cfg.bitrate_bps);

    let encode = thread::Builder::new()
        .name("loopback-encode".into())
        .spawn(move || -> Result<(u64, u64)> {
            let mut encoder = OpenH264Encoder::new();
            // OpenH264 prints two advisory warnings here by design: frame skipping is
            // off (so the rate controller is advisory, not enforced) and scene-change
            // detection is off (so keyframes stay on-demand). Both are deliberate P1
            // choices — rate enforcement is T-0403. See HANDOFF 2026-11-18.
            encoder.configure(&EncoderConfig {
                codec: CodecKind::H264,
                width,
                height,
                fps,
                bitrate_bps,
            })?;

            let mut encoded_count = 0u64;
            let mut dropped = 0u64;
            while let Ok(item) = enc_rx.recv() {
                let started = Instant::now();
                // Single IDR at the start: the GOP is infinite by design (T-0102), and
                // the loopback has no loss, so no recovery keyframes are needed here.
                let encoded =
                    encoder.encode(&item.frame.data, item.frame.capture_ts_us, item.index == 0)?;
                let encode_us = dur_us(started.elapsed());
                encoded_count += 1;
                let out = EncodedItem {
                    index: item.index,
                    encoded,
                    capture_us: item.capture_us,
                    encode_us,
                };
                match enc_tx.send_latest(out) {
                    Ok(true) => dropped += 1,
                    Ok(false) => {}
                    Err(_) => break,
                }
            }
            Ok((encoded_count, dropped))
        })
        .map_err(|e| Error::Other(format!("loopback: spawn encode: {e}")))?;

    let decode = thread::Builder::new()
        .name("loopback-decode".into())
        .spawn(move || -> Result<(u64, u64)> {
            let mut decoder = OpenH264Decoder::new();
            let mut decoded_count = 0u64;
            let mut dropped = 0u64;
            while let Ok(item) = dec_rx.recv() {
                let started = Instant::now();
                let decoded = decoder.decode(&item.encoded.data, item.encoded.is_keyframe)?;
                let decode_us = dur_us(started.elapsed());
                decoded_count += 1;
                let out = DecodedItem {
                    index: item.index,
                    decoded,
                    capture_us: item.capture_us,
                    encode_us: item.encode_us,
                    decode_us,
                };
                match dec_tx.send_latest(out) {
                    Ok(true) => dropped += 1,
                    Ok(false) => {}
                    Err(_) => break,
                }
            }
            Ok((decoded_count, dropped))
        })
        .map_err(|e| Error::Other(format!("loopback: spawn decode: {e}")))?;

    let present = thread::Builder::new()
        .name("loopback-present".into())
        .spawn(move || -> Result<PresentOutcome> {
            let mut outcome = PresentOutcome::default();
            // `out_rx` ends when decode drops its sender, so this loop terminates on
            // shutdown rather than needing a sentinel.
            while let Ok(item) = out_rx.recv() {
                let started = Instant::now();
                // Present stand-in until T-0105: validate the frame the viewer would
                // receive, then account for it. This is not a texture upload.
                if item.decoded.width != width || item.decoded.height != height {
                    return Err(Error::Other(format!(
                        "loopback: present got {}x{}, expected {width}x{height}",
                        item.decoded.width, item.decoded.height
                    )));
                }
                let present_us = dur_us(started.elapsed());

                let timings = StageTimings {
                    capture: item.capture_us,
                    encode: item.encode_us,
                    decode: item.decode_us,
                    present: present_us,
                };
                outcome.timings.accumulate(timings);
                outcome.max_frame_total_us = outcome.max_frame_total_us.max(timings.total_us());
                outcome.presented += 1;
                outcome.last_index = Some(item.index);
            }
            Ok(outcome)
        })
        .map_err(|e| Error::Other(format!("loopback: spawn present: {e}")))?;

    // Join in pipeline order. A failed stage breaks the chain, the downstream stages
    // see their channel close (or their sends return Err) and end on their own.
    let (frames_captured, dropped_before_encode) =
        capture.join().map_err(|e| thread_error("capture", &e))??;
    let (frames_encoded, dropped_before_decode) =
        encode.join().map_err(|e| thread_error("encode", &e))??;
    let (frames_decoded, dropped_before_present) =
        decode.join().map_err(|e| thread_error("decode", &e))??;
    let outcome = present.join().map_err(|e| thread_error("present", &e))??;

    Ok(RunStats {
        frames_captured,
        frames_encoded,
        frames_decoded,
        frames_presented: outcome.presented,
        dropped_before_encode,
        dropped_before_decode,
        dropped_before_present,
        timings: outcome.timings,
        max_frame_total_us: outcome.max_frame_total_us,
        last_presented_index: outcome.last_index,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_channel_evicts_oldest_and_never_blocks() {
        for cap in [1usize, 2] {
            let (tx, rx) = latest_channel::<u32>(cap);
            let mut dropped = 0u64;
            for i in 0..100u32 {
                if tx.send_latest(i).expect("receiver is alive") {
                    dropped += 1;
                }
            }

            let drained: Vec<u32> = rx.try_iter().collect();
            let kept = u32::try_from(cap).expect("cap fits u32");
            assert_eq!(drained.len(), cap, "capacity {cap}");
            assert_eq!(
                drained,
                ((100 - kept)..100).collect::<Vec<_>>(),
                "capacity {cap}: the newest items must survive"
            );
            assert_eq!(dropped, 100 - u64::from(kept), "capacity {cap}");
        }
    }

    #[test]
    fn send_latest_hands_the_value_back_when_consumer_is_gone() {
        let (tx, rx) = latest_channel::<u32>(1);
        drop(rx);
        assert_eq!(tx.send_latest(7), Err(7));
    }

    fn small_config() -> PipelineConfig {
        PipelineConfig {
            fps: 30,
            width: 64,
            height: 48,
            bitrate_bps: 500_000,
            frames: 12,
            queue_cap: 2,
        }
    }

    #[test]
    fn rejects_invalid_config_before_spawning_threads() {
        let base = small_config();
        let cases = [
            PipelineConfig { width: 0, ..base },
            PipelineConfig { height: 0, ..base },
            PipelineConfig { width: 65, ..base },
            PipelineConfig { height: 49, ..base },
            PipelineConfig { fps: 0, ..base },
            PipelineConfig {
                bitrate_bps: 0,
                ..base
            },
            PipelineConfig { frames: 0, ..base },
            PipelineConfig {
                queue_cap: 0,
                ..base
            },
            PipelineConfig {
                queue_cap: 3,
                ..base
            },
        ];
        for cfg in cases {
            let err = run(cfg).expect_err("config must be rejected");
            assert!(matches!(err, Error::InvalidArgument(_)), "got {err:?}");
        }
    }

    #[test]
    fn pipeline_carries_every_frame_in_order() {
        let stats = run(small_config()).expect("pipeline run");

        assert_eq!(stats.frames_captured, 12);
        assert_eq!(stats.frames_encoded, 12);
        assert_eq!(stats.frames_decoded, 12);
        assert_eq!(stats.frames_presented, 12);
        assert_eq!(stats.dropped_total(), 0, "consumers kept up");
        assert_eq!(stats.last_presented_index, Some(11));
        assert!(
            stats.timings.capture > 0 && stats.timings.encode > 0 && stats.timings.decode > 0,
            "each stage must report real cost: {:?}",
            stats.timings
        );
        assert!(stats.timings.total_us() >= stats.max_frame_total_us);
        assert_eq!(stats.mean_frame_total_us(), stats.timings.total_us() / 12);
    }
}
