//! Host agent binary (stub). Capture→encode→send pipeline lands in P1–P2.
#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use vd_common::init_tracing;

/// remote_viewer host agent.
#[derive(Debug, Parser)]
#[command(name = "vd-agent", about = "remote_viewer host agent")]
struct Args {
    /// Optional signal server URL (unused in P0).
    #[arg(long)]
    signal: Option<String>,
}

fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();
    tracing::info!(
        signal = args.signal.as_deref().unwrap_or("(none)"),
        "vd-agent stub — pipeline not started (P0)"
    );
    println!(
        "vd-agent {} — stub OK (remote_viewer)",
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}
