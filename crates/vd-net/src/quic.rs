//! QUIC endpoint setup on top of `quinn` (T-0202).
//!
//! Scope: bind → accept (host) and pinned-certificate connect (viewer), with an
//! explicit idle timeout. Control and Input mapping is [`crate::streams`] ([T-0203]);
//! datagrams are [T-0204]. This module stops at a working connection plus raw
//! bidirectional streams.
//!
//! # TLS posture (dev)
//!
//! The server generates a self-signed certificate (SANs `localhost`, `vd-local`) per
//! process. The viewer must be given that certificate **out of band** and it is pinned
//! in a rustls root store — there is no "skip verification" mode here. Until P3
//! (signalling, ADR-0004/Q1) distributes real identity material, the certificate is a
//! development trust anchor, not a product one.
//!
//! # Runtime
//!
//! `quinn`'s endpoint needs an async I/O driver. All entry points must be called from
//! within a Tokio runtime (`#[tokio::main]`, `#[tokio::test]`); `vd-net` itself stays
//! sync and does not own a runtime.
//!
//! [T-0203]: ../../../docs/07_TASKS.md
//! [T-0204]: ../../../docs/07_TASKS.md

use std::net::SocketAddr;
use std::sync::{Arc, Once};
use std::time::Duration;

use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use quinn::{Connection, Endpoint, IdleTimeout, RecvStream, SendStream, TransportConfig, VarInt};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use vd_common::{Error, Result};

/// ALPN protocol identifier. Both ends must offer exactly this.
pub const ALPN: &[u8] = b"vd/1";

/// Idle timeout used when the caller does not override it.
///
/// 10 s is long enough to survive a slow handshake or a paused debugger, short enough
/// that a vanished peer is detected before TCP's default keepalive would notice.
pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(10);

/// QUIC knobs exposed for P2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuicConfig {
    /// Close the connection after this much time without traffic from the peer.
    pub idle_timeout: Duration,
    /// Quinn outgoing datagram buffer (bytes). Older unsent datagrams are dropped when
    /// full — that is the latest-frame-wins behaviour for video under congestion.
    pub datagram_send_buffer_size: usize,
    /// Quinn incoming datagram buffer (bytes). `0` disables receiving datagrams.
    pub datagram_receive_buffer_size: usize,
}

impl Default for QuicConfig {
    fn default() -> Self {
        Self {
            idle_timeout: DEFAULT_IDLE_TIMEOUT,
            datagram_send_buffer_size: crate::datagram::DEFAULT_DATAGRAM_SEND_BUFFER,
            datagram_receive_buffer_size: crate::datagram::DEFAULT_DATAGRAM_RECV_BUFFER,
        }
    }
}

impl QuicConfig {
    fn validate(&self) -> Result<()> {
        if self.idle_timeout.is_zero() {
            return Err(Error::InvalidArgument(
                "quic: idle_timeout must be non-zero".into(),
            ));
        }
        if self.datagram_send_buffer_size == 0 {
            return Err(Error::InvalidArgument(
                "quic: datagram_send_buffer_size must be non-zero".into(),
            ));
        }
        Ok(())
    }

    fn transport(self) -> Result<Arc<TransportConfig>> {
        self.validate()?;
        let idle = IdleTimeout::try_from(self.idle_timeout).map_err(|_| {
            Error::InvalidArgument(format!(
                "quic: idle_timeout {:?} does not fit a QUIC VarInt (milliseconds)",
                self.idle_timeout
            ))
        })?;
        let mut transport = TransportConfig::default();
        transport.max_idle_timeout(Some(idle));
        transport.datagram_send_buffer_size(self.datagram_send_buffer_size);
        let recv = if self.datagram_receive_buffer_size == 0 {
            None
        } else {
            Some(self.datagram_receive_buffer_size)
        };
        transport.datagram_receive_buffer_size(recv);
        Ok(Arc::new(transport))
    }
}

/// Install the ring provider once per process.
///
/// `rustls` 0.23 needs a process-wide default crypto provider when configs are built
/// through `builder()`. Failure means another provider is already installed, which is
/// fine — the builders then use that one.
fn install_crypto_provider() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// Self-signed development identity for one server process.
struct Identity {
    cert: CertificateDer<'static>,
    key: PrivateKeyDer<'static>,
}

fn dev_identity() -> Result<Identity> {
    let generated =
        rcgen::generate_simple_self_signed(vec!["localhost".to_owned(), "vd-local".to_owned()])
            .map_err(|e| Error::Crypto(format!("quic: generate self-signed certificate: {e}")))?;
    let cert = generated.cert.der().clone();
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(generated.key_pair.serialize_der()));
    Ok(Identity { cert, key })
}

/// A bound QUIC server endpoint.
#[derive(Debug)]
pub struct QuicServer {
    endpoint: Endpoint,
    local_addr: SocketAddr,
    cert: CertificateDer<'static>,
    idle_timeout: Duration,
}

impl QuicServer {
    /// Bind with [`QuicConfig::default`].
    ///
    /// # Errors
    /// Socket bind failure, TLS/QUIC configuration failure, or no Tokio runtime.
    pub fn bind(addr: SocketAddr) -> Result<Self> {
        Self::bind_with(addr, QuicConfig::default())
    }

    /// Bind with explicit knobs.
    ///
    /// Must be called from within a Tokio runtime: `quinn` spawns its UDP driver on
    /// the current one.
    ///
    /// # Errors
    /// Invalid config, socket bind failure, TLS/QUIC configuration failure, or no
    /// Tokio runtime.
    pub fn bind_with(addr: SocketAddr, cfg: QuicConfig) -> Result<Self> {
        install_crypto_provider();
        let identity = dev_identity()?;

        let mut tls = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![identity.cert.clone()], identity.key)
            .map_err(|e| Error::Crypto(format!("quic: server TLS config: {e}")))?;
        tls.alpn_protocols = vec![ALPN.to_vec()];
        let crypto = QuicServerConfig::try_from(tls)
            .map_err(|e| Error::Crypto(format!("quic: server QUIC config: {e}")))?;

        let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(crypto));
        server_config.transport_config(cfg.transport()?);

        let endpoint = Endpoint::server(server_config, addr)
            .map_err(|e| Error::Io(format!("quic: bind {addr}: {e}")))?;
        let local_addr = endpoint
            .local_addr()
            .map_err(|e| Error::Io(format!("quic: local address: {e}")))?;
        tracing::info!(%local_addr, ?cfg, "quic server bound");
        Ok(Self {
            endpoint,
            local_addr,
            cert: identity.cert,
            idle_timeout: cfg.idle_timeout,
        })
    }

    /// Address the socket is actually bound to (resolves port 0).
    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// DER certificate the client must pin.
    #[must_use]
    pub fn cert_der(&self) -> &[u8] {
        self.cert.as_ref()
    }

    /// Idle timeout applied to accepted connections.
    #[must_use]
    pub fn idle_timeout(&self) -> Duration {
        self.idle_timeout
    }

    /// Wait for the next peer and complete the QUIC handshake.
    ///
    /// # Errors
    /// Endpoint closed, or the handshake/TLS exchange failed (e.g. the client did not
    /// pin this server's certificate or used a different ALPN).
    pub async fn accept(&self) -> Result<QuicConnection> {
        let incoming = self.endpoint.accept().await.ok_or_else(|| {
            Error::Io("quic: endpoint closed while waiting for a connection".into())
        })?;
        let conn = incoming
            .await
            .map_err(|e| Error::Io(format!("quic: accept handshake: {e}")))?;
        tracing::info!(peer = %conn.remote_address(), "quic connection accepted");
        Ok(QuicConnection { inner: conn })
    }

    /// Close the endpoint and every connection on it.
    pub fn close(&self) {
        self.endpoint.close(VarInt::from_u32(0), b"shutdown");
    }
}

/// A connected QUIC peer plus the client endpoint that owns its I/O driver.
#[derive(Debug)]
pub struct QuicClient {
    endpoint: Endpoint,
    connection: QuicConnection,
}

impl QuicClient {
    /// Connect to `addr` and pin the server to `server_cert_der`.
    ///
    /// `server_name` must be covered by the certificate's SANs; the development
    /// identity includes `localhost` and `vd-local`.
    ///
    /// # Errors
    /// Invalid config, socket/TLS setup failure, connection refused, handshake failure
    /// (certificate mismatch, unknown name, ALPN mismatch), or no Tokio runtime.
    pub async fn connect(
        addr: SocketAddr,
        server_name: &str,
        server_cert_der: &[u8],
        cfg: QuicConfig,
    ) -> Result<Self> {
        install_crypto_provider();
        cfg.validate()?;

        let pinned = CertificateDer::from(server_cert_der.to_vec());
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(pinned)
            .map_err(|e| Error::Crypto(format!("quic: pinned certificate rejected: {e}")))?;
        let mut tls = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        tls.alpn_protocols = vec![ALPN.to_vec()];
        let crypto = QuicClientConfig::try_from(tls)
            .map_err(|e| Error::Crypto(format!("quic: client QUIC config: {e}")))?;

        let mut client_config = quinn::ClientConfig::new(Arc::new(crypto));
        client_config.transport_config(cfg.transport()?);

        let bind: SocketAddr = if addr.is_ipv4() {
            SocketAddr::from(([0, 0, 0, 0], 0))
        } else {
            SocketAddr::from(([0u16; 8], 0))
        };
        let mut endpoint =
            Endpoint::client(bind).map_err(|e| Error::Io(format!("quic: client socket: {e}")))?;
        endpoint.set_default_client_config(client_config);

        let connecting = endpoint
            .connect(addr, server_name)
            .map_err(|e| Error::Io(format!("quic: connect {addr}: {e}")))?;
        let conn = connecting
            .await
            .map_err(|e| Error::Io(format!("quic: handshake with {addr}: {e}")))?;
        tracing::info!(peer = %addr, %server_name, "quic client connected");
        Ok(Self {
            endpoint,
            connection: QuicConnection { inner: conn },
        })
    }

    /// The established connection.
    #[must_use]
    pub fn connection(&self) -> &QuicConnection {
        &self.connection
    }

    /// Local UDP address in use.
    #[must_use]
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.endpoint.local_addr().ok()
    }

    /// Close the connection (the endpoint is dropped with this value).
    pub fn close(&self, code: u32, reason: &str) {
        self.connection.close(code, reason);
    }
}

/// One QUIC connection; cheap to clone, safe to use from tasks.
#[derive(Debug, Clone)]
pub struct QuicConnection {
    inner: Connection,
}

impl QuicConnection {
    /// Peer address.
    #[must_use]
    pub fn remote_address(&self) -> SocketAddr {
        self.inner.remote_address()
    }

    /// Close with an application error code and reason.
    pub fn close(&self, code: u32, reason: &str) {
        self.inner.close(VarInt::from_u32(code), reason.as_bytes());
    }

    /// Why the connection closed, if it has.
    #[must_use]
    pub fn close_reason(&self) -> Option<String> {
        self.inner.close_reason().map(|e| e.to_string())
    }

    /// Open one raw bidirectional stream.
    ///
    /// Prefer [`crate::streams::ReliableChannels`] for Control and Input.
    ///
    /// # Errors
    /// Connection closed or stream limits exhausted.
    pub async fn open_bi(&self) -> Result<(SendStream, RecvStream)> {
        self.inner
            .open_bi()
            .await
            .map_err(|e| Error::Io(format!("quic: open bidirectional stream: {e}")))
    }

    /// Accept a bidirectional stream opened by the peer.
    ///
    /// # Errors
    /// Connection closed, including when the peer stops opening new streams.
    pub async fn accept_bi(&self) -> Result<(SendStream, RecvStream)> {
        self.inner
            .accept_bi()
            .await
            .map_err(|e| Error::Io(format!("quic: accept bidirectional stream: {e}")))
    }

    /// Escape hatch for advanced quinn features. Prefer
    /// [`crate::streams::ReliableChannels`] and [`crate::datagram::DatagramIo`].
    #[must_use]
    pub fn inner(&self) -> &Connection {
        &self.inner
    }
}
