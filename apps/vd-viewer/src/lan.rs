//! Direct LAN viewer session (no signal server) — T-0208.
//!
//! Connects to a host with a pinned certificate, opens Control/Input streams, then
//! receives Video datagrams → reassembles → decodes on a worker thread. The UI only
//! drains the newest RGBA frame (same contract as [`crate::loopback::LoopbackSource`]).

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use tokio::runtime::Builder as RuntimeBuilder;
use vd_codec::{DecodedFrame, Decoder, OpenH264Decoder};
use vd_common::{Error, Result};
use vd_net::datagram::DatagramIo;
use vd_net::quic::{QuicClient, QuicConfig};
use vd_net::reassemble::FrameReassembler;
use vd_net::streams::ReliableChannels;
use vd_proto::{Channel, VideoFrameFragment};

/// Knobs for connecting to a LAN host.
#[derive(Debug, Clone)]
pub struct LanClientConfig {
    /// Host QUIC address.
    pub connect: SocketAddr,
    /// DER certificate to pin (written by `vd-agent --cert-out`).
    pub cert: PathBuf,
    /// TLS server name covered by the cert SANs (`localhost` / `vd-local`).
    pub server_name: String,
}

impl LanClientConfig {
    /// # Errors
    /// Empty server name or missing cert file.
    pub fn validate(&self) -> Result<()> {
        if self.server_name.is_empty() {
            return Err(Error::InvalidArgument(
                "lan viewer: server-name must be non-empty".into(),
            ));
        }
        if !self.cert.is_file() {
            return Err(Error::InvalidArgument(format!(
                "lan viewer: cert file not found: {}",
                self.cert.display()
            )));
        }
        Ok(())
    }
}

/// Network video source with the same drain API as the loopback worker.
#[derive(Debug)]
pub struct LanSource {
    frames: Receiver<DecodedFrame>,
    stop: Arc<AtomicBool>,
    dropped: Arc<AtomicU64>,
    produced: Arc<AtomicU64>,
    last_error: Arc<Mutex<Option<String>>>,
    worker: Option<JoinHandle<()>>,
}

impl LanSource {
    /// Connect and start receiving.
    ///
    /// # Errors
    /// Invalid config or failure to spawn the worker.
    pub fn start(cfg: LanClientConfig) -> Result<Self> {
        cfg.validate()?;
        let (tx, rx) = bounded::<DecodedFrame>(1);
        let stop = Arc::new(AtomicBool::new(false));
        let dropped = Arc::new(AtomicU64::new(0));
        let produced = Arc::new(AtomicU64::new(0));
        let last_error = Arc::new(Mutex::new(None));

        let worker = {
            let stop = Arc::clone(&stop);
            let dropped = Arc::clone(&dropped);
            let produced = Arc::clone(&produced);
            let last_error = Arc::clone(&last_error);
            let worker_rx = rx.clone();
            thread::Builder::new()
                .name("vd-viewer-lan".into())
                .spawn(move || {
                    let result = run_lan_worker(cfg, &tx, &worker_rx, &stop, &dropped, &produced);
                    if let Err(e) = result {
                        if let Ok(mut slot) = last_error.lock() {
                            *slot = Some(e.to_string());
                        }
                    }
                })
                .map_err(|e| Error::Other(format!("lan viewer: spawn worker: {e}")))?
        };

        Ok(Self {
            frames: rx,
            stop,
            dropped,
            produced,
            last_error,
            worker: Some(worker),
        })
    }

    /// Newest decoded frame; older queued frames count as dropped.
    #[must_use]
    pub fn try_recv_latest(&self) -> Option<DecodedFrame> {
        let mut newest = self.frames.try_recv().ok()?;
        while let Ok(frame) = self.frames.try_recv() {
            newest = frame;
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
        Some(newest)
    }

    #[must_use]
    pub fn produced(&self) -> u64 {
        self.produced.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn last_error(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|slot| slot.clone())
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.worker.as_ref().is_none_or(JoinHandle::is_finished)
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for LanSource {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_lan_worker(
    cfg: LanClientConfig,
    tx: &Sender<DecodedFrame>,
    worker_rx: &Receiver<DecodedFrame>,
    stop: &AtomicBool,
    dropped: &AtomicU64,
    produced: &AtomicU64,
) -> Result<()> {
    let rt = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Other(format!("lan viewer: tokio runtime: {e}")))?;
    rt.block_on(async_lan(cfg, tx, worker_rx, stop, dropped, produced))
}

async fn async_lan(
    cfg: LanClientConfig,
    tx: &Sender<DecodedFrame>,
    worker_rx: &Receiver<DecodedFrame>,
    stop: &AtomicBool,
    dropped: &AtomicU64,
    produced: &AtomicU64,
) -> Result<()> {
    let cert = std::fs::read(Path::new(&cfg.cert))
        .map_err(|e| Error::Io(format!("lan viewer: read cert: {e}")))?;
    let client = QuicClient::connect(cfg.connect, &cfg.server_name, &cert, QuicConfig::default())
        .await
        .map_err(|e| Error::Io(format!("lan viewer: connect: {e}")))?;
    tracing::info!(peer = %cfg.connect, "lan viewer connected");

    let _streams = ReliableChannels::open(client.connection())
        .await
        .map_err(|e| Error::Io(format!("lan viewer: open streams: {e}")))?;
    let dgrams = DatagramIo::new(client.connection().clone());
    let mut reasm = FrameReassembler::new();
    let mut decoder = OpenH264Decoder::new();

    while !stop.load(Ordering::Acquire) {
        let recv = tokio::time::timeout(Duration::from_millis(200), dgrams.recv());
        let Ok(outcome) = recv.await else {
            continue;
        };
        let (channel, payload) = outcome.map_err(|e| Error::Io(format!("lan datagram: {e}")))?;
        if channel != Channel::Video {
            continue;
        }
        let frag: VideoFrameFragment = postcard::from_bytes(&payload)
            .map_err(|e| Error::Protocol(format!("lan viewer: bad fragment: {e}")))?;
        let Some(assembled) = reasm
            .push(frag)
            .map_err(|e| Error::Protocol(format!("lan reassembly: {e}")))?
        else {
            continue;
        };
        let decoded = decoder
            .decode(&assembled.data, assembled.is_keyframe)
            .map_err(|e| Error::Other(format!("lan decode: {e}")))?;
        publish(tx, worker_rx, dropped, produced, decoded);
    }

    client.close(0, "viewer stopped");
    Ok(())
}

fn publish(
    tx: &Sender<DecodedFrame>,
    worker_rx: &Receiver<DecodedFrame>,
    dropped: &AtomicU64,
    produced: &AtomicU64,
    frame: DecodedFrame,
) {
    match tx.try_send(frame) {
        Ok(()) => {
            produced.fetch_add(1, Ordering::Relaxed);
        }
        Err(TrySendError::Full(frame)) => {
            let _ = worker_rx.try_recv();
            dropped.fetch_add(1, Ordering::Relaxed);
            match tx.try_send(frame) {
                Ok(()) => {
                    produced.fetch_add(1, Ordering::Relaxed);
                }
                Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                    dropped.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        Err(TrySendError::Disconnected(_)) => {}
    }
}
