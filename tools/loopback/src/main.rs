//! `vd-loopback` — throwaway P1 harness: capture→encode→decode→present in one process.
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
}

fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();
    let cfg = PipelineConfig {
        fps: args.fps,
        width: args.width,
        height: args.height,
        ..PipelineConfig::default()
    };
    tracing::info!(?cfg, "starting loopback pipeline");
    match vd_loopback::run(cfg) {
        Ok(()) => Ok(()),
        Err(err) => {
            println!("vd-loopback stub: not implemented yet (T-0104): {err}");
            Ok(())
        }
    }
}
