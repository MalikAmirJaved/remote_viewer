//! Single-process capture → encode → decode → present pipeline (P1 loopback).
//!
//! Public surface so `tools/vd-bench` can measure per-stage latency (T-0106) without
//! importing a product binary. The staged thread pipeline itself lands in T-0104.
//!
//! Design: [docs/01_ARCHITECTURE.md](../../../docs/01_ARCHITECTURE.md) §2 —
//! bounded `crossbeam-channel` per stage, capacity 1–2, latest-frame-wins.
#![forbid(unsafe_code)]

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
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            fps: 30,
            width: 1280,
            height: 720,
            bitrate_bps: 4_000_000,
        }
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
}

/// Run the loopback pipeline until stopped.
///
/// # Errors
/// Always [`Error::NotImplemented`] until T-0104 wires the staged threads.
pub fn run(_cfg: PipelineConfig) -> Result<()> {
    Err(Error::NotImplemented("P1 loopback pipeline (T-0104)"))
}
