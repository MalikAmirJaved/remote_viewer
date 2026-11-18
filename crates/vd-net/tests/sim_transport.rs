//! Integration tests for the [`Transport`] contract using the in-memory simulator.
//!
//! These pin the behaviour every future backend (quinn in T-0202/T-0203/T-0204) must
//! honour: channel preservation, direction independence, close semantics, and clean
//! rejection of malformed frames.

mod common;

use bytes::Bytes;
use common::{decode_packet, encode_packet, SimPair};
use vd_common::Error;
use vd_net::{SendMode, Transport};
use vd_netsim::LinkConfig;
use vd_proto::Channel;

fn wake(pair: &mut SimPair) {
    // LAN profile has zero latency, but a tick is still required to deliver.
    pair.advance_ms(0);
}

#[test]
fn send_recv_roundtrip_in_both_directions() {
    let mut pair = SimPair::new(LinkConfig::lan(), LinkConfig::lan(), 42);

    pair.a
        .send(
            Channel::Control,
            SendMode::Reliable,
            Bytes::from_static(b"hello b"),
        )
        .expect("a send");
    pair.b
        .send(
            Channel::Input,
            SendMode::Reliable,
            Bytes::from_static(b"hello a"),
        )
        .expect("b send");
    wake(&mut pair);

    assert_eq!(
        pair.b.try_recv().expect("b recv"),
        Some((Channel::Control, Bytes::from_static(b"hello b")))
    );
    assert_eq!(
        pair.a.try_recv().expect("a recv"),
        Some((Channel::Input, Bytes::from_static(b"hello a")))
    );

    // Nothing else is queued in either direction.
    assert!(pair.a.try_recv().expect("a empty").is_none());
    assert!(pair.b.try_recv().expect("b empty").is_none());
}

#[test]
fn every_channel_survives_the_adapter_in_order() {
    let mut pair = SimPair::new(LinkConfig::lan(), LinkConfig::lan(), 7);
    let channels = [
        Channel::Control,
        Channel::Input,
        Channel::InputMove,
        Channel::Video,
        Channel::Cursor,
        Channel::Audio,
        Channel::Feedback,
        Channel::Clipboard,
        Channel::File,
        Channel::Chat,
    ];

    for (i, ch) in channels.iter().enumerate() {
        pair.a
            .send(
                *ch,
                SendMode::Datagram,
                Bytes::from(vec![u8::try_from(i).unwrap(); 3]),
            )
            .expect("send");
    }
    wake(&mut pair);

    for (i, ch) in channels.iter().enumerate() {
        let (got_ch, payload) = pair.b.try_recv().expect("recv").expect("frame present");
        assert_eq!(got_ch, *ch, "channel tag {i} must round-trip");
        assert_eq!(payload.len(), 3);
    }
    assert!(pair.b.try_recv().expect("empty").is_none());
}

#[test]
fn latency_defers_delivery_until_time_advances() {
    let cfg = LinkConfig {
        latency_ms: 50,
        ..LinkConfig::default()
    };
    let mut pair = SimPair::new(cfg.clone(), cfg, 99);

    pair.a
        .send(
            Channel::Video,
            SendMode::Datagram,
            Bytes::from_static(b"frag"),
        )
        .expect("send");
    pair.advance_ms(49);
    assert!(
        pair.b.try_recv().expect("recv").is_none(),
        "must not arrive before one-way latency"
    );

    pair.advance_ms(1);
    assert_eq!(
        pair.b.try_recv().expect("recv"),
        Some((Channel::Video, Bytes::from_static(b"frag")))
    );
}

#[test]
fn total_loss_delivers_nothing() {
    let cfg = LinkConfig {
        loss_pct: 100,
        ..LinkConfig::default()
    };
    let mut pair = SimPair::new(cfg.clone(), cfg, 3);

    for seq in 0u8..16 {
        pair.a
            .send(Channel::Video, SendMode::Datagram, Bytes::from(vec![seq]))
            .expect("send is accepted even when the link drops it");
    }
    pair.advance_ms(10);

    assert!(pair.b.try_recv().expect("recv").is_none());
}

#[test]
fn base_latency_does_not_leak_between_directions() {
    let mut pair = SimPair::new(
        LinkConfig {
            latency_ms: 40,
            ..LinkConfig::default()
        },
        LinkConfig::lan(),
        11,
    );

    pair.a
        .send(
            Channel::Control,
            SendMode::Reliable,
            Bytes::from_static(b"slow"),
        )
        .expect("send");
    pair.b
        .send(
            Channel::Control,
            SendMode::Reliable,
            Bytes::from_static(b"fast"),
        )
        .expect("send");

    // Reverse direction has no added latency, forward direction still in flight.
    pair.advance_ms(0);
    assert_eq!(
        pair.a.try_recv().expect("a recv"),
        Some((Channel::Control, Bytes::from_static(b"fast")))
    );
    assert!(pair.b.try_recv().expect("b recv").is_none());

    pair.advance_ms(40);
    assert_eq!(
        pair.b.try_recv().expect("b recv"),
        Some((Channel::Control, Bytes::from_static(b"slow")))
    );
}

#[test]
fn closed_transport_rejects_send_and_local_addr_is_stable() {
    let mut pair = SimPair::new(LinkConfig::lan(), LinkConfig::lan(), 5);

    assert_eq!(pair.a.local_addr().expect("addr"), "sim://a");
    assert_eq!(pair.b.local_addr().expect("addr"), "sim://b");

    pair.a.close().expect("close");
    let err = pair
        .a
        .send(Channel::Chat, SendMode::Reliable, Bytes::from_static(b"x"))
        .expect_err("closed transport must reject send");
    assert!(matches!(err, Error::Io(_)), "got {err:?}");

    // Closing one side does not break the other.
    pair.b
        .send(Channel::Chat, SendMode::Reliable, Bytes::from_static(b"ok"))
        .expect("b still open");
    wake(&mut pair);
    assert_eq!(
        pair.a.try_recv().expect("a recv"),
        Some((Channel::Chat, Bytes::from_static(b"ok")))
    );
}

#[test]
fn malformed_frames_are_protocol_errors_not_panics() {
    assert!(matches!(
        decode_packet(&Bytes::new()),
        Err(Error::Protocol(_))
    ));
    assert!(matches!(
        decode_packet(&Bytes::from_static(&[200])),
        Err(Error::Protocol(_))
    ));

    // Round-trip sanity for the encoder used by the adapter.
    let frame = encode_packet(Channel::Feedback, b"rr");
    assert_eq!(
        decode_packet(&frame).expect("decode"),
        (Channel::Feedback, Bytes::from_static(b"rr"))
    );
}
