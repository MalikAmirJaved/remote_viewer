//! Video reassembly + netsim loss (T-0206).
//!
//! Module unit tests cover the reassembler itself; this file drives fragments through
//! `vd-netsim` and the shared `SimPair` harness.

mod common;

use bytes::Bytes;
use vd_common::Error;
use vd_net::reassemble::FrameReassembler;
use vd_net::video::{fragment_frame, EncodedFrameRef, FragmentConfig};
use vd_net::{SendMode, Transport};
use vd_netsim::LinkConfig;
use vd_proto::{Channel, Envelope, Payload, VideoFrameFragment, PROTOCOL_VERSION};

use common::SimPair;

fn sample_frame(frame_id: u64, data: &[u8]) -> EncodedFrameRef<'_> {
    EncodedFrameRef {
        frame_id,
        is_keyframe: frame_id == 1,
        capture_ts_us: frame_id * 10_000,
        codec: "h264",
        width: 64,
        height: 48,
        data,
    }
}

fn encode_frag(frag: &VideoFrameFragment, seq: u64) -> Bytes {
    let env = Envelope {
        version: PROTOCOL_VERSION,
        channel: Channel::Video,
        seq,
        ts_us: frag.capture_ts_us,
        payload: Payload::Video(frag.clone()),
    };
    Bytes::from(env.encode().expect("encode fragment envelope"))
}

fn decode_frag(payload: &[u8]) -> VideoFrameFragment {
    let env = Envelope::decode(payload).expect("decode envelope");
    match env.payload {
        Payload::Video(frag) => frag,
        other => panic!("expected Video payload, got {other:?}"),
    }
}

#[test]
fn fragment_then_reassemble_round_trip() {
    let data = vec![0x5Au8; 2400]; // > 2 × 1150 → three fragments at default cap
    let frags = fragment_frame(sample_frame(3, &data), FragmentConfig::default()).expect("frag");
    assert!(frags.len() >= 2);

    let mut reasm = FrameReassembler::new();
    let mut done = None;
    // Reverse order to prove reassembly is order-independent.
    for frag in frags.into_iter().rev() {
        if let Some(frame) = reasm.push(frag).expect("push") {
            done = Some(frame);
        }
    }
    let frame = done.expect("must complete");
    assert_eq!(frame.frame_id, 3);
    assert_eq!(frame.data, data);
    assert_eq!(frame.width, 64);
    assert_eq!(frame.height, 48);
}

#[test]
fn netsim_delivers_out_of_order_fragments_that_still_complete() {
    let data = b"abcdefghijklmnopqrstuvwxyz0123456789".repeat(40); // multi-fragment
    let frags = fragment_frame(
        sample_frame(1, &data),
        FragmentConfig::with_max_data_len(32),
    )
    .expect("frag");
    assert!(frags.len() > 2);

    let mut pair = SimPair::new(
        LinkConfig {
            latency_ms: 5,
            reorder_pct: 80,
            loss_pct: 0,
            ..LinkConfig::default()
        },
        LinkConfig::default(),
        0xDEAD_BEEF,
    );

    for (i, frag) in frags.iter().enumerate() {
        pair.a
            .send(
                Channel::Video,
                SendMode::Datagram,
                encode_frag(frag, i as u64),
            )
            .expect("send");
    }
    pair.advance_ms(50);

    let mut reasm = FrameReassembler::new();
    let mut completed = None;
    while let Some((ch, payload)) = pair.b.try_recv().expect("recv") {
        assert_eq!(ch, Channel::Video);
        if let Some(frame) = reasm.push(decode_frag(&payload)).expect("push") {
            completed = Some(frame);
        }
    }
    let frame = completed.expect("reordered fragments must still complete");
    assert_eq!(frame.data, data.as_slice());
    assert_eq!(reasm.stats().completed, 1);
}

#[test]
fn netsim_loss_leaves_frame_incomplete_until_newer_wins() {
    // Frame 1: three small fragments. Send them through a lossy link; with a high
    // loss rate and a fixed seed at least one is dropped so the frame stays open.
    let frags_old = fragment_frame(
        sample_frame(1, b"frame-one-payload-aaa"),
        FragmentConfig::with_max_data_len(8),
    )
    .expect("frag old");
    assert!(frags_old.len() >= 3);

    let mut lost_at_least_one = false;
    let mut reasm = FrameReassembler::new();
    // Try a few seeds so the loss model actually drops something (deterministic LCG).
    for seed in 1u64..64 {
        let mut pair = SimPair::new(
            LinkConfig {
                latency_ms: 1,
                loss_pct: 50,
                ..LinkConfig::default()
            },
            LinkConfig::default(),
            seed,
        );
        for (i, frag) in frags_old.iter().enumerate() {
            pair.a
                .send(
                    Channel::Video,
                    SendMode::Datagram,
                    encode_frag(frag, i as u64),
                )
                .expect("send");
        }
        pair.advance_ms(20);

        let mut got = 0usize;
        while let Some((ch, payload)) = pair.b.try_recv().expect("recv") {
            assert_eq!(ch, Channel::Video);
            assert!(reasm.push(decode_frag(&payload)).expect("push").is_none());
            got += 1;
        }
        if got > 0 && got < frags_old.len() {
            lost_at_least_one = true;
            break;
        }
        // Reset for the next seed attempt.
        reasm = FrameReassembler::new();
    }
    assert!(
        lost_at_least_one,
        "expected netsim to drop at least one fragment of frame 1"
    );
    assert_eq!(reasm.in_progress_count(), 1);
    assert_eq!(reasm.stats().completed, 0);

    // Frame 2 arrives complete over a clean path — latest-frame-wins abandons frame 1.
    let mut clean = SimPair::new(LinkConfig::lan(), LinkConfig::lan(), 99);
    let frags_new = fragment_frame(
        sample_frame(2, b"frame-two-complete"),
        FragmentConfig::with_max_data_len(8),
    )
    .expect("frag new");
    for (i, frag) in frags_new.iter().enumerate() {
        clean
            .a
            .send(
                Channel::Video,
                SendMode::Datagram,
                encode_frag(frag, 100 + i as u64),
            )
            .expect("send new");
    }
    clean.advance_ms(5);

    let mut completed = None;
    while let Some((_, payload)) = clean.b.try_recv().expect("recv new") {
        if let Some(frame) = reasm.push(decode_frag(&payload)).expect("push new") {
            completed = Some(frame);
        }
    }
    let frame = completed.expect("frame 2 must complete");
    assert_eq!(frame.frame_id, 2);
    assert_eq!(frame.data, b"frame-two-complete");
    assert!(reasm.stats().abandoned >= 1);
    assert_eq!(reasm.in_progress_count(), 0);
}

#[test]
fn netsim_total_loss_on_old_frame_still_allows_newer() {
    let mut pair = SimPair::new(
        LinkConfig {
            loss_pct: 100,
            ..LinkConfig::default()
        },
        LinkConfig::default(),
        7,
    );
    let old = fragment_frame(sample_frame(1, b"gone"), FragmentConfig::default()).expect("old");
    for (i, frag) in old.iter().enumerate() {
        pair.a
            .send(
                Channel::Video,
                SendMode::Datagram,
                encode_frag(frag, i as u64),
            )
            .expect("send");
    }
    pair.advance_ms(10);
    assert!(pair.b.try_recv().expect("recv").is_none());

    // Switch to a lossless pair for the newer frame; same reassembler instance.
    let mut reasm = FrameReassembler::new();
    let mut clean = SimPair::new(LinkConfig::lan(), LinkConfig::lan(), 8);
    let new = fragment_frame(sample_frame(2, b"ok"), FragmentConfig::default()).expect("new");
    for (i, frag) in new.iter().enumerate() {
        clean
            .a
            .send(
                Channel::Video,
                SendMode::Datagram,
                encode_frag(frag, 10 + i as u64),
            )
            .expect("send");
    }
    clean.advance_ms(5);
    let mut done = None;
    while let Some((_, payload)) = clean.b.try_recv().expect("recv") {
        if let Some(frame) = reasm.push(decode_frag(&payload)).expect("push") {
            done = Some(frame);
        }
    }
    assert_eq!(done.expect("frame 2").data, b"ok");
}

#[test]
fn invalid_envelope_is_not_the_reassemblers_job() {
    // Sanity: Transport decode of empty payload still surfaces as protocol at the edge.
    let err = Envelope::decode(&[]).expect_err("empty");
    assert!(matches!(err, Error::Protocol(_)));
}
