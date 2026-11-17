//! Closed-loop rate control: quality ladder + estimator skeleton (pure logic, no IO).
//!
//! See [`docs/03_NETWORK_ADAPTATION.md`](../../docs/03_NETWORK_ADAPTATION.md).
#![forbid(unsafe_code)]

mod controller;
mod ladder;

pub use controller::{RateController, RateControllerConfig, RateDecision, ReceiverStats};
pub use ladder::{QualityLadder, QualityLevel, QualityMode};

/// Trait for pluggable rate controllers (agent-side).
pub trait RateControl {
    /// Ingest a receiver report and produce a new encoding/network decision.
    fn on_receiver_stats(&mut self, stats: &ReceiverStats) -> RateDecision;

    /// Current quality level index into the ladder.
    fn level_index(&self) -> usize;
}
