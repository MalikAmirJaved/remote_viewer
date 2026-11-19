//! Minimal two-process QUIC check for T-0202.
//!
//! Terminal 1: `cargo run -p vd-net --example quic_ping -- server --cert target/quic-dev-cert.der`
//! Terminal 2: `cargo run -p vd-net --example quic_ping -- client --cert target/quic-dev-cert.der`
//!
//! The server prints its bound address and writes its self-signed DER certificate for
//! the client to pin. It exits after one echo. This is a development trust anchor, not
//! the product identity story (P3 signalling).

use std::net::SocketAddr;
use std::process::ExitCode;

use vd_net::quic::{QuicClient, QuicConfig, QuicServer};

const DEFAULT_ADDR: &str = "127.0.0.1:9443";
const DEFAULT_CERT: &str = "target/quic-dev-cert.der";

fn usage() -> ! {
    eprintln!(
        "usage:\n  quic_ping server [--bind {DEFAULT_ADDR}] [--cert {DEFAULT_CERT}]\n  quic_ping client [--addr {DEFAULT_ADDR}] [--cert {DEFAULT_CERT}]"
    );
    std::process::exit(2);
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().unwrap_or_else(|| usage());

    let mut addr = DEFAULT_ADDR.to_owned();
    let mut cert_path = DEFAULT_CERT.to_owned();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--bind" | "--addr" => addr = args.next().unwrap_or_else(|| usage()),
            "--cert" => cert_path = args.next().unwrap_or_else(|| usage()),
            other => {
                eprintln!("unknown argument: {other}");
                usage();
            }
        }
    }
    let addr: SocketAddr = addr.parse()?;

    match mode.as_str() {
        "server" => {
            let server = QuicServer::bind(addr)?;
            if let Some(parent) = std::path::Path::new(&cert_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&cert_path, server.cert_der())?;
            println!(
                "listening on {} — certificate written to {cert_path} (pin it on the client)",
                server.local_addr()
            );
            let conn = server.accept().await?;
            println!("accepted connection from {}", conn.remote_address());
            let (mut send, mut recv) = conn.accept_bi().await?;
            let request = recv.read_to_end(1024).await?;
            println!(
                "received {} bytes: {}",
                request.len(),
                String::from_utf8_lossy(&request)
            );
            send.write_all(b"pong from vd-net")
                .await
                .map_err(|e| format!("write: {e}"))?;
            send.finish().map_err(|e| format!("finish: {e}"))?;
            // Give the peer a moment to read the response before the endpoint closes.
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            server.close();
            println!("server done");
        }
        "client" => {
            let cert = std::fs::read(&cert_path)?;
            let client =
                QuicClient::connect(addr, "localhost", &cert, QuicConfig::default()).await?;
            println!(
                "connected to {} from {:?}",
                client.connection().remote_address(),
                client.local_addr()
            );
            let (mut send, mut recv) = client.connection().open_bi().await?;
            send.write_all(b"ping from vd-net")
                .await
                .map_err(|e| format!("write: {e}"))?;
            send.finish().map_err(|e| format!("finish: {e}"))?;
            let mut reply = [0u8; 16];
            let n = recv
                .read(&mut reply)
                .await?
                .ok_or("server closed the stream without a reply")?;
            println!("reply: {}", String::from_utf8_lossy(&reply[..n]));
            client.close(0, "done");
        }
        other => {
            eprintln!("unknown mode: {other}");
            usage();
        }
    }
    Ok(())
}
