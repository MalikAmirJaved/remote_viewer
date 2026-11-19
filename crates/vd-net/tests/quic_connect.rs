//! QUIC connect/accept contract (T-0202).
//!
//! These are real QUIC handshakes over UDP loopback, not mocks: quinn runs its full
//! TLS 1.3 exchange, certificate pinning and ALPN check on every test.

use std::net::SocketAddr;
use std::time::Duration;

use vd_common::Error;
use vd_net::quic::{QuicClient, QuicConfig, QuicServer};

fn loopback() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 0))
}

#[tokio::test]
async fn client_and_server_connect_and_exchange_a_stream() {
    let server = QuicServer::bind(loopback()).expect("bind server");
    let addr = server.local_addr();
    let cert = server.cert_der().to_vec();

    let accept = tokio::spawn(async move {
        let conn = server.accept().await.expect("accept");
        let (mut send, mut recv) = conn.accept_bi().await.expect("accept stream");
        let request = recv.read_to_end(1024).await.expect("read request");
        send.write_all(b"pong").await.expect("write response");
        send.finish().expect("finish response");
        (conn.remote_address(), request)
    });

    let client = QuicClient::connect(addr, "localhost", &cert, QuicConfig::default())
        .await
        .expect("connect");
    let (mut send, _recv) = client.connection().open_bi().await.expect("open stream");
    send.write_all(b"ping").await.expect("write request");
    send.finish().expect("finish request");

    let (peer, request) = accept.await.expect("join accept task");
    assert_eq!(request, b"ping");
    // The server sees the client's ephemeral source port, not the server's own port.
    assert_eq!(peer.ip(), addr.ip());
    assert_ne!(peer.port(), 0);
    assert_eq!(client.connection().remote_address(), addr);
    assert_eq!(client.connection().close_reason(), None);
}

#[tokio::test]
async fn an_unpinned_certificate_is_rejected() {
    let server = QuicServer::bind(loopback()).expect("bind server");
    let addr = server.local_addr();
    // The server must drive the handshake, otherwise quinn waits for its own idle
    // timeout instead of surfacing the TLS failure.
    let accept = tokio::spawn(async move { server.accept().await });

    // A second, unrelated self-signed identity: exactly what pinning must refuse.
    let other = QuicServer::bind(loopback()).expect("bind other server");
    let wrong_cert = other.cert_der().to_vec();

    let outcome = tokio::time::timeout(
        Duration::from_secs(5),
        QuicClient::connect(addr, "localhost", &wrong_cert, QuicConfig::default()),
    )
    .await;

    match outcome {
        Ok(Err(Error::Io(_))) => {}
        Ok(Err(other)) => panic!("expected an I/O handshake error, got {other:?}"),
        Ok(Ok(_)) => panic!("connected with an unpinned certificate"),
        Err(elapsed) => panic!("handshake did not fail within 5 s: {elapsed}"),
    }
    assert!(
        accept.await.expect("join accept").is_err(),
        "server must reject the unpinned handshake"
    );
}

#[tokio::test]
async fn unknown_server_name_is_rejected() {
    let server = QuicServer::bind(loopback()).expect("bind server");
    let addr = server.local_addr();
    let cert = server.cert_der().to_vec();
    let accept = tokio::spawn(async move { server.accept().await });

    let outcome = tokio::time::timeout(
        Duration::from_secs(5),
        QuicClient::connect(
            addr,
            "not-in-the-cert.example",
            &cert,
            QuicConfig::default(),
        ),
    )
    .await;

    match outcome {
        Ok(Err(Error::Io(_))) => {}
        Ok(Err(other)) => panic!("expected an I/O handshake error, got {other:?}"),
        Ok(Ok(_)) => panic!("connected with a server name the certificate does not cover"),
        Err(elapsed) => panic!("handshake did not fail within 5 s: {elapsed}"),
    }
    let _ = accept.await.expect("join accept");
}

#[tokio::test]
async fn idle_timeout_is_configurable_and_zero_is_rejected() {
    let cfg = QuicConfig {
        idle_timeout: Duration::from_millis(2500),
    };
    let server = QuicServer::bind_with(loopback(), cfg).expect("bind server");
    assert_eq!(server.idle_timeout(), Duration::from_millis(2500));

    assert!(matches!(
        QuicServer::bind_with(
            loopback(),
            QuicConfig {
                idle_timeout: Duration::ZERO,
            }
        ),
        Err(Error::InvalidArgument(_))
    ));

    assert!(matches!(
        QuicClient::connect(
            server.local_addr(),
            "localhost",
            server.cert_der(),
            QuicConfig {
                idle_timeout: Duration::ZERO,
            }
        )
        .await,
        Err(Error::InvalidArgument(_))
    ));
}

#[tokio::test]
async fn idle_connection_is_closed_after_the_configured_timeout() {
    let cfg = QuicConfig {
        idle_timeout: Duration::from_millis(300),
    };
    let server = QuicServer::bind_with(loopback(), cfg).expect("bind server");
    let addr = server.local_addr();
    let cert = server.cert_der().to_vec();

    let accept = tokio::spawn(async move { server.accept().await });
    let client = QuicClient::connect(addr, "localhost", &cert, cfg)
        .await
        .expect("connect");
    let accepted = accept.await.expect("join accept").expect("accept");

    // Both ends go silent; QUIC must give up within a few multiples of 300 ms.
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert!(
        client.connection().close_reason().is_some() || accepted.close_reason().is_some(),
        "idle connection was not closed by the idle timeout"
    );
}
