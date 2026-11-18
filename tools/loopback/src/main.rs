//! `vd-loopback` — P1 harness: capture→encode→decode→present in one process.
#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use vd_common::init_tracing;
use vd_loopback::PipelineConfig;

/// Loopback pipeline demo.
#[derive(Debug, Parser)]
#[command(
    name = "vd-loopback",
    about = "remote_viewer single-process pipeline (P1)"
)]
struct Args {
    /// Target frames per second.
    #[arg(long, default_value_t = 30)]
    fps: u8,
    /// Pipeline width.
    #[arg(long, default_value_t = 1280)]
    width: u32,
    /// Pipeline height.
    #[arg(long, default_value_t = 720)]
    height: u32,
    /// Frames to push through before exiting.
    #[arg(long, default_value_t = 300)]
    frames: u32,
    /// Per-stage queue capacity (1 or 2).
    #[arg(long, default_value_t = 2)]
    queue_cap: usize,
}

fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();
    let cfg = PipelineConfig {
        fps: args.fps,
        width: args.width,
        height: args.height,
        frames: args.frames,
        queue_cap: args.queue_cap,
        ..PipelineConfig::default()
    };
    tracing::info!(?cfg, "starting loopback pipeline");
    let stats = vd_loopback::run(cfg)?;

    println!(
        "captured={} encoded={} decoded={} presented={} dropped={}",
        stats.frames_captured,
        stats.frames_encoded,
        stats.frames_decoded,
        stats.frames_presented,
        stats.dropped_total()
    );
    println!(
        "per-stage total us: capture={} encode={} decode={} present={}",
        stats.timings.capture, stats.timings.encode, stats.timings.decode, stats.timings.present
    );
    println!(
        "mean frame total us={} max frame total us={} last index={:?}",
        stats.mean_frame_total_us(),
        stats.max_frame_total_us,
        stats.last_presented_index
    );
    Ok(())
}
