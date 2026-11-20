//! Host agent — local stub or direct LAN stream (T-0208).
#![forbid(unsafe_code)]

mod lan;

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use vd_common::init_tracing;

/// remote_viewer host agent.
#[derive(Debug, Parser)]
#[command(name = "vd-agent", about = "remote_viewer host agent")]
struct Args {
    /// Optional signal server URL (unused until P3).
    #[arg(long)]
    signal: Option<String>,

    /// Listen for a direct LAN viewer (QUIC). Enables the T-0208 demo path.
    #[arg(long)]
    listen: Option<SocketAddr>,

    /// Write the pinned server certificate (DER) here for the viewer `--cert` flag.
    #[arg(long, default_value = "vd-agent-cert.der")]
    cert_out: PathBuf,

    /// Fake-capture width (even).
    #[arg(long, default_value_t = 960)]
    width: u32,

    /// Fake-capture height (even).
    #[arg(long, default_value_t = 540)]
    height: u32,

    /// Fake-capture frames per second.
    #[arg(long, default_value_t = 30)]
    fps: u8,

    /// Encoder / pacer bitrate in bits per second.
    #[arg(long, default_value_t = 2_500_000)]
    bitrate: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();

    if let Some(listen) = args.listen {
        return lan::run(lan::LanHostConfig {
            listen,
            cert_out: args.cert_out,
            width: args.width,
            height: args.height,
            fps: args.fps,
            bitrate_bps: args.bitrate,
        })
        .await;
    }

    tracing::info!(
        signal = args.signal.as_deref().unwrap_or("(none)"),
        "vd-agent stub — pass --listen <addr> for the LAN demo (T-0208)"
    );
    println!(
        "vd-agent {} — stub OK. LAN demo: --listen 127.0.0.1:7856",
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_declares_listen_and_cert_out() {
        let mut cmd = Args::command();
        let help = cmd.render_long_help().to_string();
        assert!(help.contains("--listen"));
        assert!(help.contains("--cert-out"));
    }
}
