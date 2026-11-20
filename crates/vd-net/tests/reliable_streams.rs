//! Control and Input on separate QUIC streams (T-0203).
//!
//! Real handshakes over UDP loopback, same harness as `quic_connect`.

use std::net::SocketAddr;
use std::time::Duration;

use vd_common::Error;
use vd_net::quic::{QuicClient, QuicConfig, QuicConnection, QuicServer};
use vd_net::streams::{
    ReliableChannels, CONTROL_STREAM_PRIORITY, INPUT_STREAM_PRIORITY, MAX_RELIABLE_PAYLOAD,
};
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
async fn control_and_input_round_trip_on_separate_streams() {
    let (client, server_conn) = connect_pair().await;
    let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();

    let accepted = tokio::spawn(async move {
        let mut streams = ReliableChannels::accept(&server_conn)
            .await
            .expect("accept streams");
        assert_eq!(
            streams
                .send_priority(Channel::Control)
                .expect("control priority"),
            CONTROL_STREAM_PRIORITY
        );
        assert_eq!(
            streams
                .send_priority(Channel::Input)
                .expect("input priority"),
            INPUT_STREAM_PRIORITY
        );
        assert!(
            streams.send_priority(Channel::Input).expect("input")
                > streams.send_priority(Channel::Control).expect("control")
        );

        let c1 = streams.recv(Channel::Control).await.expect("recv c1");
        let i1 = streams.recv(Channel::Input).await.expect("recv i1");
        let c2 = streams.recv(Channel::Control).await.expect("recv c2");
        streams
            .send(Channel::Control, b"ack-c")
            .await
            .expect("send ack-c");
        streams
            .send(Channel::Input, b"ack-i")
            .await
            .expect("send ack-i");
        // Keep the connection up until the client has read the replies. Dropping the
        // endpoint first can finish the send halves before those bytes are read.
        release_rx.await.expect("client released the server");
        (c1, i1, c2)
    });

    let mut streams = ReliableChannels::open(client.connection())
        .await
        .expect("open streams");
    assert_eq!(
        streams
            .send_priority(Channel::Input)
            .expect("input priority"),
        INPUT_STREAM_PRIORITY
    );
    assert!(
        streams.send_priority(Channel::Input).expect("input")
            > streams.send_priority(Channel::Control).expect("control")
    );

    streams
        .send(Channel::Control, b"c1")
        .await
        .expect("send c1");
    streams.send(Channel::Input, b"i1").await.expect("send i1");
    streams
        .send(Channel::Control, b"")
        .await
        .expect("send empty control");

    let ack_c = streams.recv(Channel::Control).await.expect("recv ack-c");
    let ack_i = streams.recv(Channel::Input).await.expect("recv ack-i");
    release_tx.send(()).expect("release server");

    let (c1, i1, c2) = accepted.await.expect("join");
    assert_eq!(c1, b"c1");
    assert_eq!(i1, b"i1");
    assert_eq!(c2, b"");
    assert_eq!(ack_c, b"ack-c");
    assert_eq!(ack_i, b"ack-i");
}

#[tokio::test]
async fn oversized_and_non_reliable_channels_are_rejected() {
    let (client, server_conn) = connect_pair().await;
    let accepted = tokio::spawn(async move {
        let mut streams = ReliableChannels::accept(&server_conn)
            .await
            .expect("accept streams");
        streams.recv(Channel::Input).await.expect("recv input")
    });

    let mut streams = ReliableChannels::open(client.connection())
        .await
        .expect("open streams");

    let video = streams
        .send(Channel::Video, b"nope")
        .await
        .expect_err("video");
    match video {
        Error::InvalidArgument(msg) => assert!(msg.contains("Video"), "{msg}"),
        other => panic!("expected InvalidArgument, got {other:?}"),
    }
    let feedback = streams.recv(Channel::Feedback).await.expect_err("feedback");
    match feedback {
        Error::InvalidArgument(msg) => assert!(msg.contains("Feedback"), "{msg}"),
        other => panic!("expected InvalidArgument, got {other:?}"),
    }

    let big = vec![0u8; MAX_RELIABLE_PAYLOAD + 1];
    let oversized = streams
        .send(Channel::Input, &big)
        .await
        .expect_err("oversized");
    match oversized {
        Error::InvalidArgument(msg) => {
            assert!(
                msg.contains(&(MAX_RELIABLE_PAYLOAD + 1).to_string()),
                "{msg}"
            );
        }
        other => panic!("expected InvalidArgument, got {other:?}"),
    }

    streams
        .send(Channel::Input, b"still-here")
        .await
        .expect("small input");
    let got = tokio::time::timeout(Duration::from_secs(5), accepted)
        .await
        .expect("recv did not time out")
        .expect("join");
    assert_eq!(got, b"still-here");
}

#[tokio::test]
async fn an_unknown_stream_tag_is_rejected() {
    let (client, server_conn) = connect_pair().await;
    let accepted = tokio::spawn(async move { ReliableChannels::accept(&server_conn).await });

    let (mut send, _recv) = client.connection().open_bi().await.expect("open raw");
    send.write_all(&[0xFF]).await.expect("write bad tag");

    let err = tokio::time::timeout(Duration::from_secs(5), accepted)
        .await
        .expect("accept did not time out")
        .expect("join")
        .expect_err("bad tag");
    match err {
        Error::Protocol(msg) => assert!(msg.contains("255") || msg.contains("tag"), "{msg}"),
        other => panic!("expected Protocol, got {other:?}"),
    }
}

#[tokio::test]
async fn a_duplicate_channel_tag_is_rejected() {
    let (client, server_conn) = connect_pair().await;
    let accepted = tokio::spawn(async move { ReliableChannels::accept(&server_conn).await });

    for _ in 0..2 {
        let (mut send, _recv) = client.connection().open_bi().await.expect("open raw");
        send.write_all(&[Channel::Control as u8])
            .await
            .expect("write control tag");
    }

    let err = tokio::time::timeout(Duration::from_secs(5), accepted)
        .await
        .expect("accept did not time out")
        .expect("join")
        .expect_err("duplicate");
    match err {
        Error::Protocol(msg) => assert!(msg.contains("duplicate"), "{msg}"),
        other => panic!("expected Protocol, got {other:?}"),
    }
}

#[tokio::test]
async fn a_frame_longer_than_the_cap_is_rejected() {
    let (client, server_conn) = connect_pair().await;
    let accepted = tokio::spawn(async move {
        let mut streams = ReliableChannels::accept(&server_conn)
            .await
            .expect("accept streams");
        streams.recv(Channel::Control).await
    });

    let (mut control, _recv) = client.connection().open_bi().await.expect("open control");
    control
        .write_all(&[Channel::Control as u8])
        .await
        .expect("tag control");
    let declared = (MAX_RELIABLE_PAYLOAD as u32) + 1;
    control
        .write_all(&declared.to_be_bytes())
        .await
        .expect("write length");

    let (mut input, _recv) = client.connection().open_bi().await.expect("open input");
    input
        .write_all(&[Channel::Input as u8])
        .await
        .expect("tag input");

    let err = tokio::time::timeout(Duration::from_secs(5), accepted)
        .await
        .expect("recv did not time out")
        .expect("join")
        .expect_err("long frame");
    match err {
        Error::Protocol(msg) => assert!(msg.contains("exceeds"), "{msg}"),
        other => panic!("expected Protocol, got {other:?}"),
    }
}
