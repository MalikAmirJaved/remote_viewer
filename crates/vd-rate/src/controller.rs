//! Rate controller state machine skeleton.

use crate::ladder::{QualityLadder, QualityLevel, QualityMode};
use crate::RateControl;
use vd_common::Error;

/// Snapshot of viewer-side reception quality.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReceiverStats {
    /// Received bitrate (bps).
    pub recv_bps: u32,
    /// Loss in parts-per-million.
    pub loss_ppm: u32,
    /// Jitter µs.
    pub jitter_us: u32,
    /// One-way delay gradient (signed).
    pub owd_gradient: i32,
    /// Decode time ms.
    pub decode_ms: u16,
    /// Last frame id.
    pub last_frame_id: u64,
    /// Viewer requests IDR.
    pub need_idr: bool,
    /// Simulated time in ms (for hysteresis timers in tests).
    pub now_ms: u64,
}

/// Encoding / pacing decision produced by the controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateDecision {
    /// Selected ladder level.
    pub level: QualityLevel,
    /// Index into the ladder.
    pub level_index: usize,
    /// Target bitrate bps.
    pub target_bps: u32,
    /// Force IDR on next frame.
    pub force_idr: bool,
    /// Suggested FEC overhead ratio in percent (0–50).
    pub fec_percent: u8,
}

/// Tunables for the skeleton controller.
#[derive(Debug, Clone)]
pub struct RateControllerConfig {
    /// Loss ppm above which we step down immediately.
    pub down_loss_ppm: u32,
    /// OWD gradient above which we treat path as congested.
    pub down_owd_gradient: i32,
    /// Minimum ms between up-shifts (slow up).
    pub up_hold_ms: u64,
    /// Minimum ms between down-shifts (fast down still respects this small floor).
    pub down_hold_ms: u64,
    /// Loss ppm below which up-shift is allowed.
    pub up_loss_ppm: u32,
    /// Quality mode preference.
    pub mode: QualityMode,
}

impl Default for RateControllerConfig {
    fn default() -> Self {
        Self {
            down_loss_ppm: 20_000, // 2 %
            down_owd_gradient: 5,
            up_hold_ms: 2_500,
            down_hold_ms: 100,
            up_loss_ppm: 5_000, // 0.5 %
            mode: QualityMode::TextSharp,
        }
    }
}

/// Pure state-machine rate controller (GCC/BBR-like estimator comes in later tasks).
#[derive(Debug, Clone)]
pub struct RateController {
    ladder: QualityLadder,
    cfg: RateControllerConfig,
    index: usize,
    last_change_ms: u64,
}

impl RateController {
    /// Create at the top of the ladder.
    #[must_use]
    pub fn new(ladder: QualityLadder, cfg: RateControllerConfig) -> Self {
        Self {
            ladder,
            cfg,
            index: 0,
            last_change_ms: 0,
        }
    }

    /// Current decision without updating state.
    pub fn current(&self) -> Result<RateDecision, Error> {
        let level = *self
            .ladder
            .get(self.index)
            .ok_or(Error::InvalidArgument("empty quality ladder".into()))?;
        Ok(RateDecision {
            level,
            level_index: self.index,
            target_bps: level.target_bps,
            force_idr: false,
            fec_percent: 0,
        })
    }

    fn fec_for_loss(loss_ppm: u32) -> u8 {
        // Simple mapping: 0% loss → 0 FEC; 5% → ~20%; cap 40%.
        let pct = loss_ppm / 2_500; // 10_000 ppm = 4%
        u8::try_from(pct.min(40)).unwrap_or(40)
    }
}

impl RateControl for RateController {
    fn on_receiver_stats(&mut self, stats: &ReceiverStats) -> RateDecision {
        let congested = stats.loss_ppm >= self.cfg.down_loss_ppm
            || stats.owd_gradient >= self.cfg.down_owd_gradient;
        let clear = stats.loss_ppm <= self.cfg.up_loss_ppm
            && stats.owd_gradient < self.cfg.down_owd_gradient;

        let mut force_idr = stats.need_idr;
        let elapsed = stats.now_ms.saturating_sub(self.last_change_ms);

        if congested && elapsed >= self.cfg.down_hold_ms {
            let next = (self.index + 1).min(self.ladder.len().saturating_sub(1));
            if next != self.index {
                self.index = next;
                self.last_change_ms = stats.now_ms;
                force_idr = true; // resolution/fps change → IDR
            }
        } else if clear && elapsed >= self.cfg.up_hold_ms && self.index > 0 {
            self.index -= 1;
            self.last_change_ms = stats.now_ms;
            force_idr = true;
        }

        let level = *self.ladder.get(self.index).expect("index in range");
        // Mode currently recorded for future rung selection bias; skeleton keeps ladder order.
        let _ = self.cfg.mode;

        RateDecision {
            level,
            level_index: self.index,
            target_bps: level.target_bps,
            force_idr,
            fec_percent: Self::fec_for_loss(stats.loss_ppm),
        }
    }

    fn level_index(&self) -> usize {
        self.index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(loss_ppm: u32, owd: i32, now_ms: u64) -> ReceiverStats {
        ReceiverStats {
            recv_bps: 1_000_000,
            loss_ppm,
            jitter_us: 0,
            owd_gradient: owd,
            decode_ms: 4,
            last_frame_id: 1,
            need_idr: false,
            now_ms,
        }
    }

    #[test]
    fn downs_on_loss() {
        let mut c = RateController::new(QualityLadder::standard(), RateControllerConfig::default());
        assert_eq!(c.level_index(), 0);
        let d = c.on_receiver_stats(&stats(50_000, 0, 200));
        assert!(d.level_index >= 1, "expected downshift on high loss");
        assert!(d.force_idr);
    }

    #[test]
    fn holds_when_stable() {
        let mut c = RateController::new(QualityLadder::standard(), RateControllerConfig::default());
        let _ = c.on_receiver_stats(&stats(0, 0, 100));
        let d = c.on_receiver_stats(&stats(1_000, 0, 500));
        assert_eq!(d.level_index, 0, "should hold at top when healthy");
    }

    #[test]
    fn slow_up_after_recovery() {
        let cfg = RateControllerConfig {
            up_hold_ms: 2_500,
            ..RateControllerConfig::default()
        };
        let mut c = RateController::new(QualityLadder::standard(), cfg);

        // Force down
        let _ = c.on_receiver_stats(&stats(50_000, 0, 100));
        assert!(c.level_index() >= 1);

        // Clear but too soon to up-shift
        let d_early = c.on_receiver_stats(&stats(0, 0, 500));
        assert_eq!(d_early.level_index, c.level_index());

        let idx_before = c.level_index();
        let d_late = c.on_receiver_stats(&stats(0, 0, 100 + 2_500));
        assert_eq!(d_late.level_index, idx_before - 1, "slow up after hold");
    }
}
