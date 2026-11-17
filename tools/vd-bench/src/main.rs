//! Benchmark harness stub — records glass-to-glass latency once pipeline exists.
#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use vd_common::{init_tracing, MonotonicClock};
use vd_netsim::{LinkConfig, SimLink};

/// remote_viewer bench harness.
#[derive(Debug, Parser)]
#[command(name = "vd-bench", about = "remote_viewer latency harness")]
struct Args {
    /// Network profile name: lan|wan-weak.
    #[arg(long, default_value = "lan")]
    profile: String,
}

fn main() -> Result<()> {
    init_tracing();
    let args = Args::parse();
    let cfg = match args.profile.as_str() {
        "wan-weak" => LinkConfig::wan_weak(),
        _ => LinkConfig::lan(),
    };
    let mut link = SimLink::new(cfg, 1);
    let clock = MonotonicClock::new();
    let t0 = clock.now_us();
    link.send(b"probe".as_slice())?;
    link.advance_ms(u64::from(LinkConfig::lan().latency_ms));
    let n = link.recv_ready().len();
    let t1 = clock.now_us();
    println!(
        "vd-bench stub: profile={} delivered={} elapsed_us={} (not a real glass-to-glass measurement)",
        args.profile,
        n,
        t1.saturating_duration_since(t0)
    );
    Ok(())
}
