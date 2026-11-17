//! Viewer binary (stub). egui UI and decode path land in later phases.
#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use vd_common::init_tracing;

/// remote_viewer viewer client.
#[derive(Debug, Parser)]
#[command(name = "vd-viewer", about = "remote_viewer viewer")]
struct Args {
    /// Peer ID to connect to (unused in P0).
    #[arg(long)]
    peer: Option<String>,
}

fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();
    tracing::info!(
        peer = args.peer.as_deref().unwrap_or("(none)"),
        "vd-viewer stub — UI not started (P0)"
    );
    println!(
        "vd-viewer {} — stub OK (remote_viewer)",
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}
