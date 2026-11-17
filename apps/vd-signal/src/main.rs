//! Signaling server stub (axum HTTP/WebSocket in P3).
#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use vd_common::init_tracing;

/// remote_viewer signaling server.
#[derive(Debug, Parser)]
#[command(name = "vd-signal", about = "remote_viewer signaling server")]
struct Args {
    /// Listen address.
    #[arg(long, default_value = "0.0.0.0:7700")]
    listen: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();
    tracing::info!(%args.listen, "vd-signal stub — not listening yet (P0)");
    println!(
        "vd-signal {} — stub OK (remote_viewer)",
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}
