//! Direct LAN host session (no signal server) — T-0208.
//!
//! Binds a QUIC server, writes the pinned development certificate for the viewer,
//! accepts one peer, then streams fake-capture → H.264 → paced video datagrams.
//! Control/Input streams are accepted so the viewer can open them; payload use is
//! later work.

use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use tokio::sync::watch;
use vd_capture::{Capturer, FakeCaptureConfig, FakeCapturer};
use vd_codec::{CodecKind, Encoder, EncoderConfig, OpenH264Encoder};
use vd_common::MonotonicClock;
use vd_net::datagram::DatagramIo;
use vd_net::pacer::Pacer;
use vd_net::quic::{QuicConfig, QuicServer};
use vd_net::streams::ReliableChannels;
use vd_net::video::{fragment_frame, EncodedFrameRef, FragmentConfig};
use vd_proto::{Channel, VideoFrameFragment};

/// Knobs for the LAN host demo.
#[derive(Debug, Clone)]
pub struct LanHostConfig {
    /// Bind address (use port 0 to let the OS pick).
    pub listen: SocketAddr,
    /// Where to write the DER certificate the viewer must pin.
    pub cert_out: std::path::PathBuf,
    pub width: u32,
    pub height: u32,
    pub fps: u8,
    pub bitrate_bps: u32,
}

impl LanHostConfig {
    /// # Errors
    /// Zero/odd dimensions, zero fps/bitrate.
    pub fn validate(&self) -> Result<()> {
        if self.width == 0 || self.height == 0 || self.width % 2 != 0 || self.height % 2 != 0 {
            bail!("lan host: width/height must be non-zero even values");
        }
        if self.fps == 0 || self.bitrate_bps == 0 {
            bail!("lan host: fps and bitrate must be non-zero");
        }
        Ok(())
    }
}

/// Run until Ctrl+C or the peer disconnects.
pub async fn run(cfg: LanHostConfig) -> Result<()> {
    cfg.validate()?;

    let server =
        QuicServer::bind_with(cfg.listen, QuicConfig::default()).context("bind QUIC server")?;
    let addr = server.local_addr();
    write_cert(&cfg.cert_out, server.cert_der())?;

    println!();
    println!("=== remote_viewer LAN SESSION (host) ===");
    println!("listening on {addr}");
    println!("cert written to {}", cfg.cert_out.display());
    println!("viewer example:");
    println!(
        "  cargo run -p vd-viewer --release -- --connect {addr} --cert {}",
        cfg.cert_out.display()
    );
    println!("visible session — host can end anytime with Ctrl+C");
    println!("========================================");
    println!();

    tracing::info!(%addr, cert = %cfg.cert_out.display(), "waiting for viewer");

    let conn = tokio::select! {
        conn = server.accept() => conn.context("accept viewer")?,
        _ = tokio::signal::ctrl_c() => {
            println!("cancelled before a viewer connected");
            return Ok(());
        }
    };
    println!(
        "viewer connected from {} — streaming (indicator ON)",
        conn.remote_address()
    );

    let streams = ReliableChannels::accept(&conn)
        .await
        .context("accept Control/Input streams")?;
    let _streams = streams;

    let dgrams = DatagramIo::new(conn.clone());
    let (stop_tx, stop_rx) = watch::channel(false);

    let encode_cfg = cfg.clone();
    let encode = tokio::task::spawn_blocking(move || encode_loop(encode_cfg, dgrams, stop_rx));

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Ctrl+C — ending LAN session");
            let _ = stop_tx.send(true);
        }
        res = encode => {
            let _ = stop_tx.send(true);
            res.context("encode task join")??;
        }
    }

    tokio::time::sleep(Duration::from_millis(50)).await;
    conn.close(0, "host ended session");
    server.close();
    println!("session ended — indicator OFF");
    Ok(())
}

fn write_cert(path: &Path, der: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create cert dir {}", parent.display()))?;
        }
    }
    std::fs::write(path, der).with_context(|| format!("write cert {}", path.display()))?;
    Ok(())
}

#[allow(clippy::needless_pass_by_value)] // owned so `spawn_blocking` can move them in
fn encode_loop(cfg: LanHostConfig, dgrams: DatagramIo, stop: watch::Receiver<bool>) -> Result<()> {
    let mut capturer = FakeCapturer::new(FakeCaptureConfig {
        width: cfg.width,
        height: cfg.height,
        fps: cfg.fps,
    })
    .context("create fake capturer")?;
    capturer.start(0).context("start fake capturer")?;

    let mut encoder = OpenH264Encoder::new();
    encoder
        .configure(&EncoderConfig {
            codec: CodecKind::H264,
            width: cfg.width,
            height: cfg.height,
            fps: cfg.fps,
            bitrate_bps: cfg.bitrate_bps,
        })
        .context("configure encoder")?;

    let mut pacer = Pacer::new(cfg.bitrate_bps).context("create pacer")?;
    let clock = MonotonicClock::new();
    let mut frame_id = 0u64;
    // DatagramIo caps the *postcard* payload at ≤1150 B. Fragment *data* must leave
    // room for VideoFrameFragment metadata (frame_id, codec, dims, …) or send fails
    // and the peer eventually sees a QUIC idle timeout.
    const POSTCARD_META_RESERVE: usize = 64;
    let dg_cap = dgrams
        .max_payload()
        .context("datagrams not negotiated with peer")?;
    let data_cap = dg_cap.saturating_sub(POSTCARD_META_RESERVE).max(1);
    let frag_cfg = FragmentConfig::with_max_data_len(data_cap);
    tracing::info!(dg_cap, data_cap, "lan fragment sizing");

    while !*stop.borrow() {
        let Some(captured) = capturer.capture_frame().context("capture")? else {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        };

        let force_idr = frame_id == 0;
        let encoded = encoder
            .encode(&captured.data, captured.capture_ts_us, force_idr)
            .context("encode")?;
        frame_id = frame_id.saturating_add(1);

        let view = EncodedFrameRef {
            frame_id,
            is_keyframe: encoded.is_keyframe,
            capture_ts_us: encoded.capture_ts_us,
            codec: "h264",
            width: encoded.width,
            height: encoded.height,
            data: &encoded.data,
        };
        let fragments = fragment_frame(view, frag_cfg).context("fragment")?;
        for frag in fragments {
            send_fragment(&dgrams, &mut pacer, &clock, &frag)?;
        }
    }

    capturer.stop().ok();
    Ok(())
}

fn send_fragment(
    dgrams: &DatagramIo,
    pacer: &mut Pacer,
    clock: &MonotonicClock,
    frag: &VideoFrameFragment,
) -> Result<()> {
    let payload = postcard::to_allocvec(frag).context("postcard encode fragment")?;
    loop {
        let now = clock.now_us().0;
        if pacer.allow(payload.len(), now) {
            break;
        }
        let wait = pacer.wait_us(payload.len(), now).clamp(100, 5_000);
        std::thread::sleep(Duration::from_micros(wait));
    }
    dgrams
        .send(Channel::Video, &payload)
        .context("send video datagram")?;
    Ok(())
}
