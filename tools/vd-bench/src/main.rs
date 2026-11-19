//! `vd-bench` — per-stage latency report for the P1 loopback pipeline.
//!
//! Runs the real capture → encode → decode → present pipeline
//! ([`vd_loopback::run`]) and reports **p50 / p95 / p99 / max / mean per stage**.
//!
//! Honesty boundary: these are host-side pipeline costs on one machine. There is no
//! network, no viewer texture upload and no camera, so this is **not** glass-to-glass
//! latency. The glass-to-glass table in `docs/05_PERFORMANCE_BUDGET.md` stays "not
//! measured" until P2/P3 wiring exists.

#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use vd_common::init_tracing;
use vd_loopback::{PipelineConfig, RunStats, StageTimings};

/// remote_viewer latency harness.
#[derive(Debug, Parser)]
#[command(name = "vd-bench", about = "remote_viewer per-stage latency report")]
struct Args {
    /// Frame width in pixels (must be even).
    #[arg(long, default_value_t = 1280)]
    width: u32,

    /// Frame height in pixels (must be even).
    #[arg(long, default_value_t = 720)]
    height: u32,

    /// Frames per second requested from the fake source.
    #[arg(long, default_value_t = 60)]
    fps: u8,

    /// Frames to push through before the report.
    #[arg(long, default_value_t = 300)]
    frames: u32,

    /// Per-stage queue capacity (1 or 2, per architecture §2).
    #[arg(long, default_value_t = 2)]
    queue_cap: usize,

    /// Encoder target bitrate in bits per second.
    #[arg(long, default_value_t = 6_000_000)]
    bitrate: u32,
}

/// Distribution summary for one stage, in microseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Summary {
    n: usize,
    p50: u64,
    p95: u64,
    p99: u64,
    max: u64,
    mean: u64,
}

/// Nearest-rank percentile over an already sorted slice.
///
/// Nearest-rank keeps reported values to samples that actually occurred (no
/// interpolation), which is what a latency budget should be compared against.
fn percentile_us(sorted: &[u64], percentile: u32) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let n = sorted.len() as u64;
    // ceil(p/100 * n), clamped to 1..=n.
    let rank = (u64::from(percentile) * n).div_ceil(100).clamp(1, n);
    sorted[(rank - 1) as usize]
}

/// Sort once, then derive every reported number.
fn summarize(mut samples: Vec<u64>) -> Summary {
    if samples.is_empty() {
        return Summary::default();
    }
    let n = samples.len();
    let sum: u64 = samples.iter().sum();
    samples.sort_unstable();
    Summary {
        n,
        p50: percentile_us(&samples, 50),
        p95: percentile_us(&samples, 95),
        p99: percentile_us(&samples, 99),
        max: samples[n - 1],
        mean: sum / n as u64,
    }
}

/// One column per reported row, in report order.
fn stage_columns(stats: &RunStats) -> [(&'static str, Vec<u64>); 5] {
    let pick = |f: fn(&StageTimings) -> u64| stats.samples.iter().map(f).collect::<Vec<_>>();
    [
        ("capture", pick(|s| s.capture)),
        ("encode", pick(|s| s.encode)),
        ("decode", pick(|s| s.decode)),
        ("present", pick(|s| s.present)),
        ("total", pick(StageTimings::total_us)),
    ]
}

fn ms(us: u64) -> f64 {
    us as f64 / 1000.0
}

fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();
    let cfg = PipelineConfig {
        fps: args.fps,
        width: args.width,
        height: args.height,
        bitrate_bps: args.bitrate,
        frames: args.frames,
        queue_cap: args.queue_cap,
    };
    tracing::info!(?cfg, "starting loopback benchmark");
    let stats = vd_loopback::run(cfg)?;

    println!("vd-bench — host-side pipeline stage cost, NOT glass-to-glass latency");
    println!(
        "config: {}x{} @ {} fps, {} frames, queue_cap={}, bitrate={} bps",
        cfg.width, cfg.height, cfg.fps, cfg.frames, cfg.queue_cap, cfg.bitrate_bps
    );
    println!(
        "frames: captured={} encoded={} decoded={} presented={} dropped={} (before encode={}, before decode={}, before present={})",
        stats.frames_captured,
        stats.frames_encoded,
        stats.frames_decoded,
        stats.frames_presented,
        stats.dropped_total(),
        stats.dropped_before_encode,
        stats.dropped_before_decode,
        stats.dropped_before_present
    );

    if stats.samples.is_empty() {
        println!("no presented frames — nothing to report");
        return Ok(());
    }

    println!();
    println!(
        "{:<10}{:>6}{:>10}{:>10}{:>10}{:>10}{:>10}",
        "stage", "n", "p50 ms", "p95 ms", "p99 ms", "max ms", "mean ms"
    );
    for (name, samples) in stage_columns(&stats) {
        let s = summarize(samples);
        println!(
            "{:<10}{:>6}{:>10.2}{:>10.2}{:>10.2}{:>10.2}{:>10.2}",
            name,
            s.n,
            ms(s.p50),
            ms(s.p95),
            ms(s.p99),
            ms(s.max),
            ms(s.mean)
        );
    }
    println!();
    println!(
        "note: `present` is the loopback stand-in (frame validation), not the viewer's texture upload;"
    );
    println!("      `total` sums the four stage samples for the same frame.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_percentiles_on_a_known_series() {
        let s = summarize((1..=100).collect());
        assert_eq!(
            (s.n, s.p50, s.p95, s.p99, s.max, s.mean),
            (100, 50, 95, 99, 100, 50)
        );
    }

    #[test]
    fn empty_and_single_sample_are_well_defined() {
        assert_eq!(summarize(Vec::new()), Summary::default());

        let s = summarize(vec![7]);
        assert_eq!(
            (s.n, s.p50, s.p95, s.p99, s.max, s.mean),
            (1, 7, 7, 7, 7, 7)
        );
    }

    #[test]
    fn unsorted_input_is_sorted_before_percentiles() {
        let s = summarize(vec![9, 1, 5]);
        assert_eq!((s.p50, s.p95, s.max, s.mean), (5, 9, 9, 5));
    }

    #[test]
    fn small_pipeline_run_reports_every_stage() {
        let stats = vd_loopback::run(PipelineConfig {
            fps: 30,
            width: 64,
            height: 48,
            bitrate_bps: 500_000,
            frames: 8,
            queue_cap: 2,
        })
        .expect("pipeline run");

        assert!(stats.frames_presented > 0);
        assert_eq!(stats.samples.len() as u64, stats.frames_presented);
        let columns = stage_columns(&stats);
        assert_eq!(columns.len(), 5);
        for (name, samples) in columns {
            let summary = summarize(samples);
            assert_eq!(summary.n, stats.samples.len(), "row {name}");
            // `present` is a dimension check until T-0105's texture path is wired here,
            // so it can legitimately round down to 0 µs. Every real stage must cost > 0.
            if name != "present" {
                assert!(summary.p50 > 0, "row {name} must report real cost");
            }
        }
    }
}
