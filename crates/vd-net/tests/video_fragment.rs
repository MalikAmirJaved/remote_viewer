//! T-0205 acceptance: fragmentation of a real `vd_codec::EncodedFrame` (not a stand-in).

use vd_codec::EncodedFrame;
use vd_net::video::{fragment_frame, EncodedFrameRef, FragmentConfig, DEFAULT_MAX_FRAGMENT_DATA};

fn encoded(len: usize, is_keyframe: bool) -> EncodedFrame {
    EncodedFrame {
        data: (0..len).map(|i| (i % 251) as u8).collect(),
        is_keyframe,
        capture_ts_us: 1_700_000_000_000,
        width: 1920,
        height: 1080,
    }
}

fn view(frame: &EncodedFrame, frame_id: u64) -> EncodedFrameRef<'_> {
    EncodedFrameRef {
        frame_id,
        is_keyframe: frame.is_keyframe,
        capture_ts_us: frame.capture_ts_us,
        codec: "h264",
        width: frame.width,
        height: frame.height,
        data: &frame.data,
    }
}

#[test]
fn a_real_encoded_keyframe_splits_and_concatenates_back() {
    let frame = encoded(DEFAULT_MAX_FRAGMENT_DATA * 3 + 646, true);
    let fragments =
        fragment_frame(view(&frame, 42), FragmentConfig::default()).expect("fragment frame");

    assert_eq!(fragments.len(), 4);
    assert_eq!(fragments[0].frag_total, 4);
    assert!(fragments.iter().all(|f| f.frame_id == 42));
    assert!(fragments.iter().all(|f| f.is_keyframe));
    assert!(fragments
        .iter()
        .all(|f| f.capture_ts_us == frame.capture_ts_us));
    assert!(fragments
        .iter()
        .all(|f| f.width == 1920 && f.height == 1080));
    assert!(fragments
        .iter()
        .all(|f| f.data.len() <= DEFAULT_MAX_FRAGMENT_DATA && !f.data.is_empty()));

    let joined: Vec<u8> = fragments
        .iter()
        .flat_map(|f| f.data.iter().copied())
        .collect();
    assert_eq!(joined, frame.data);
}

#[test]
fn a_delta_frame_is_one_datagram_when_it_fits() {
    let frame = encoded(900, false);
    let fragments = fragment_frame(view(&frame, 43), FragmentConfig::default()).expect("fragment");

    assert_eq!(fragments.len(), 1);
    assert_eq!(fragments[0].frag_total, 1);
    assert!(!fragments[0].is_keyframe);
    assert_eq!(fragments[0].data, frame.data);
}
