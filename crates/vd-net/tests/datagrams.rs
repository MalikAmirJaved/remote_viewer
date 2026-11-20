//! QUIC datagram path for Video / Feedback (T-0204).

use std::net::SocketAddr;
use std::time::Duration;

use vd_common::Error;
use vd_net::datagram::{DatagramIo, DEFAULT_MAX_DATAGRAM_PAYLOAD};
use vd_net::quic::{QuicClient, QuicConfig, QuicConnection, QuicServer};
use vd_proto::Channel;

fn loopback() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 0))
}

async fn connect_pair() -> (QuicClient, QuicConnection) {
    let server = QuicServer::bind(loopback()).expect("bind server");
    let addr = server.local_addr();
    let cert = server.cert_der().to_vec();
    let accepted = tokio::spawn(async move { server.accept().await.expect("accept") });
    let client = QuicClient::connect(addr, "localhost", &cert, QuicConfig::default())
        .await
        .expect("connect");
    let server_conn = accepted.await.expect("join accept");
    (client, server_conn)
}

#[tokio::test]
async fn video_and_feedback_round_trip() {
    let (client, server_conn) = connect_pair().await;
    let server = DatagramIo::new(server_conn);
    let client_io = DatagramIo::new(client.connection().clone());

    let max = client_io.max_payload().expect("datagrams enabled");
    assert!(
        max >= DEFAULT_MAX_DATAGRAM_PAYLOAD,
        "loopback max_payload {max} should cover the 1150 B fragment budget"
    );

    let received = tokio::spawn(async move {
        let (c1, p1) = server.recv().await.expect("recv video");
        let (c2, p2) = server.recv().await.expect("recv feedback");
        let (c3, p3) = server.recv().await.expect("recv empty input-move");
        (c1, p1, c2, p2, c3, p3)
    });

    let video = vec![0xABu8; 900];
    client_io.send(Channel::Video, &video).expect("send video");
    client_io
        .send(Channel::Feedback, b"rr")
        .expect("send feedback");
    client_io
        .send(Channel::InputMove, b"")
        .expect("send empty input-move");

    let (c1, p1, c2, p2, c3, p3) = tokio::time::timeout(Duration::from_secs(5), received)
        .await
        .expect("recv timed out")
        .expect("join");
    assert_eq!(c1, Channel::Video);
    assert_eq!(p1.as_ref(), video.as_slice());
    assert_eq!(c2, Channel::Feedback);
    assert_eq!(p2.as_ref(), b"rr");
    assert_eq!(c3, Channel::InputMove);
    assert!(p3.is_empty());
}

#[tokio::test]
async fn oversized_and_reliable_channels_are_rejected() {
    let (client, server_conn) = connect_pair().await;
    let _keep_alive = server_conn;
    let io = DatagramIo::new(client.connection().clone());

    let control = io.send(Channel::Control, b"nope").expect_err("control");
    match control {
        Error::InvalidArgument(msg) => assert!(msg.contains("Control"), "{msg}"),
        other => panic!("expected InvalidArgument, got {other:?}"),
    }

    let big = vec![0u8; DEFAULT_MAX_DATAGRAM_PAYLOAD + 1];
    let oversized = io.send(Channel::Video, &big).expect_err("oversized");
    match oversized {
        Error::InvalidArgument(msg) => {
            assert!(
                msg.contains(&(DEFAULT_MAX_DATAGRAM_PAYLOAD + 1).to_string()),
                "{msg}"
            );
        }
        other => panic!("expected InvalidArgument, got {other:?}"),
    }
}

#[tokio::test]
async fn oversized_reject_leaves_the_path_usable() {
    let (client, server_conn) = connect_pair().await;
    let server = DatagramIo::new(server_conn);
    let client_io = DatagramIo::new(client.connection().clone());

    let big = vec![0u8; DEFAULT_MAX_DATAGRAM_PAYLOAD + 1];
    let err = client_io.send(Channel::Video, &big).expect_err("oversized");
    assert!(matches!(err, Error::InvalidArgument(_)));

    let got = tokio::spawn(async move { server.recv().await.expect("recv") });
    client_io
        .send(Channel::Feedback, b"still")
        .expect("follow-up");
    let (channel, payload) = tokio::time::timeout(Duration::from_secs(5), got)
        .await
        .expect("timeout")
        .expect("join");
    assert_eq!(channel, Channel::Feedback);
    assert_eq!(payload.as_ref(), b"still");
}

#[tokio::test]
async fn an_unknown_datagram_tag_is_rejected() {
    let (client, server_conn) = connect_pair().await;
    let server = DatagramIo::new(server_conn);

    let received = tokio::spawn(async move { server.recv().await });

    // Bypass DatagramIo and write a raw tagged datagram with a Control tag.
    let mut raw = bytes::BytesMut::new();
    raw.extend_from_slice(&[Channel::Control as u8, b'x']);
    client
        .connection()
        .inner()
        .send_datagram(raw.freeze())
        .expect("raw send");

    let err = tokio::time::timeout(Duration::from_secs(5), received)
        .await
        .expect("timeout")
        .expect("join")
        .expect_err("bad tag");
    match err {
        Error::Protocol(msg) => {
            assert!(msg.contains("tag") || msg.contains("Control"), "{msg}");
        }
        other => panic!("expected Protocol, got {other:?}"),
    }
}

#[tokio::test]
async fn zero_send_buffer_is_rejected_at_config() {
    let cfg = QuicConfig {
        datagram_send_buffer_size: 0,
        ..QuicConfig::default()
    };
    let err = QuicServer::bind_with(loopback(), cfg).expect_err("zero send buffer");
    match err {
        Error::InvalidArgument(msg) => assert!(msg.contains("datagram_send_buffer"), "{msg}"),
        other => panic!("expected InvalidArgument, got {other:?}"),
    }
}
