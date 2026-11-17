//! cargo xtask entry — developer workflows for remote_viewer.
#![forbid(unsafe_code)]

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "remote_viewer maintainer tasks")]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// fmt + clippy -D warnings + tests
    Check,
    /// Run workspace tests
    Test,
    /// Print netem profile names
    NetemProfiles,
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.cmd {
        Cmd::Check => {
            run(&["fmt", "--all"])?;
            run(&[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ])?;
            run(&["test", "--workspace"])?;
        }
        Cmd::Test => {
            run(&["test", "--workspace"])?;
        }
        Cmd::NetemProfiles => {
            println!("lan");
            println!("wan-good");
            println!("wan-weak");
            println!("mobile-3g");
            println!("lossy-wifi");
        }
    }
    Ok(())
}

fn run(args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .args(args)
        .status()
        .with_context(|| format!("spawn cargo {}", args.join(" ")))?;
    if !status.success() {
        bail!("cargo {} failed with {status}", args.join(" "));
    }
    Ok(())
}
