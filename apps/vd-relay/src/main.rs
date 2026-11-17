//! UDP relay stub — forwards encrypted packets only (no plaintext access).
#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use vd_common::init_tracing;

/// remote_viewer UDP relay.
#[derive(Debug, Parser)]
#[command(name = "vd-relay", about = "remote_viewer UDP relay")]
struct Args {
    /// Listen address.
    #[arg(long, default_value = "0.0.0.0:7701")]
    listen: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();
    tracing::info!(%args.listen, "vd-relay stub — not forwarding yet (P0)");
    println!(
        "vd-relay {} — stub OK (remote_viewer)",
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}
