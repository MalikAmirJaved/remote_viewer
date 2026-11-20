//! Delay-gradient (trendline) + loss estimators — pure logic, no IO.
//!
//! Inspired by WebRTC GCC trendline / overuse detection, kept small and
//! deterministic for synthetic-trace unit tests.

use std::collections::VecDeque;

/// Bandwidth usage inferred from delay gradient (and optionally loss).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandwidthUsage {
    /// Path looks under capacity.
    Underuse,
    /// No strong congestion signal.
    Normal,
    /// Queues growing — back off.
    Overuse,
}

/// Combined estimate for the rate controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkEstimate {
    /// One-way-delay gradient in **µs per second** (positive ⇒ delay rising).
    pub owd_gradient_us_per_s: i32,
    /// Packet loss in parts-per-million.
    pub loss_ppm: u32,
    /// Congestion signal after overuse hysteresis.
    pub usage: BandwidthUsage,
}

/// Tunables for the delay-gradient trendline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DelayGradientConfig {
    /// Max samples kept in the regression window.
    pub window: usize,
    /// |slope| below this (µs/s) counts as Normal deadband.
    pub deadband_us_per_s: i32,
    /// Consecutive overuse observations required before [`BandwidthUsage::Overuse`].
    pub overuse_count: u8,
    /// Consecutive underuse observations required before [`BandwidthUsage::Underuse`].
    pub underuse_count: u8,
}

impl Default for DelayGradientConfig {
    fn default() -> Self {
        Self {
            window: 20,
            deadband_us_per_s: 1_000, // 1 ms/s
            overuse_count: 2,
            underuse_count: 2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DelayPoint {
    t_ms: u64,
    owd_us: i64,
}

/// OLS trendline over recent OWD samples → gradient + usage.
#[derive(Debug, Clone)]
pub struct DelayGradientEstimator {
    cfg: DelayGradientConfig,
    points: VecDeque<DelayPoint>,
    gradient_us_per_s: i32,
    overuse_streak: u8,
    underuse_streak: u8,
    usage: BandwidthUsage,
}

impl Default for DelayGradientEstimator {
    fn default() -> Self {
        Self::new(DelayGradientConfig::default())
    }
}

impl DelayGradientEstimator {
    /// Create with the given config.
    #[must_use]
    pub fn new(cfg: DelayGradientConfig) -> Self {
        let window = cfg.window.max(2);
        Self {
            cfg: DelayGradientConfig { window, ..cfg },
            points: VecDeque::with_capacity(window),
            gradient_us_per_s: 0,
            overuse_streak: 0,
            underuse_streak: 0,
            usage: BandwidthUsage::Normal,
        }
    }

    /// Current gradient (µs/s).
    #[must_use]
    pub fn gradient_us_per_s(&self) -> i32 {
        self.gradient_us_per_s
    }

    /// Current usage signal.
    #[must_use]
    pub fn usage(&self) -> BandwidthUsage {
        self.usage
    }

    /// Ingest one OWD sample at `t_ms` (monotonic) with delay `owd_us`.
    pub fn push(&mut self, t_ms: u64, owd_us: i64) -> i32 {
        if let Some(last) = self.points.back() {
            // Ignore non-monotonic clocks (keep last estimate).
            if t_ms < last.t_ms {
                return self.gradient_us_per_s;
            }
        }
        self.points.push_back(DelayPoint { t_ms, owd_us });
        while self.points.len() > self.cfg.window {
            self.points.pop_front();
        }
        self.gradient_us_per_s = fit_slope_us_per_s(&self.points);
        self.update_usage();
        self.gradient_us_per_s
    }

    /// Feed a precomputed gradient (µs/s) through the overuse detector.
    ///
    /// Used when the viewer already reports `owd_gradient` instead of raw OWD.
    pub fn observe_gradient(&mut self, gradient_us_per_s: i32) -> BandwidthUsage {
        self.gradient_us_per_s = gradient_us_per_s;
        self.update_usage();
        self.usage
    }

    fn update_usage(&mut self) {
        let g = self.gradient_us_per_s;
        let db = self.cfg.deadband_us_per_s;
        let instant = if g > db {
            BandwidthUsage::Overuse
        } else if g < -db {
            BandwidthUsage::Underuse
        } else {
            BandwidthUsage::Normal
        };

        match instant {
            BandwidthUsage::Overuse => {
                self.overuse_streak = self.overuse_streak.saturating_add(1);
                self.underuse_streak = 0;
                if self.overuse_streak >= self.cfg.overuse_count {
                    self.usage = BandwidthUsage::Overuse;
                }
            }
            BandwidthUsage::Underuse => {
                self.underuse_streak = self.underuse_streak.saturating_add(1);
                self.overuse_streak = 0;
                if self.underuse_streak >= self.cfg.underuse_count {
                    self.usage = BandwidthUsage::Underuse;
                }
            }
            BandwidthUsage::Normal => {
                self.overuse_streak = 0;
                self.underuse_streak = 0;
                self.usage = BandwidthUsage::Normal;
            }
        }
    }
}

/// Ordinary least-squares slope of OWD vs time, as µs/s.
fn fit_slope_us_per_s(points: &VecDeque<DelayPoint>) -> i32 {
    let n = points.len();
    if n < 2 {
        return 0;
    }
    let t0 = points[0].t_ms;
    // Use f64 for the fit; inputs are deterministic integers.
    let mut sum_t = 0.0_f64;
    let mut sum_d = 0.0_f64;
    let mut sum_tt = 0.0_f64;
    let mut sum_td = 0.0_f64;
    for p in points {
        let t = (p.t_ms - t0) as f64; // ms
        let d = p.owd_us as f64; // µs
        sum_t += t;
        sum_d += d;
        sum_tt += t * t;
        sum_td += t * d;
    }
    let nf = n as f64;
    let denom = nf * sum_tt - sum_t * sum_t;
    if denom.abs() < f64::EPSILON {
        return 0;
    }
    // slope in µs per ms → ×1000 = µs per s
    let slope_us_per_ms = (nf * sum_td - sum_t * sum_d) / denom;
    let slope_us_per_s = slope_us_per_ms * 1000.0;
    if !slope_us_per_s.is_finite() {
        return 0;
    }
    slope_us_per_s.round() as i32
}

/// Tunables for EWMA loss tracking.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LossEstimatorConfig {
    /// EWMA alpha in parts-per-thousand (1..=1000). Higher = faster reaction.
    pub alpha_ppt: u32,
}

impl Default for LossEstimatorConfig {
    fn default() -> Self {
        Self {
            alpha_ppt: 200, // 0.2
        }
    }
}

/// EWMA packet-loss estimator (ppm).
#[derive(Debug, Clone)]
pub struct LossEstimator {
    cfg: LossEstimatorConfig,
    loss_ppm: u32,
    initialized: bool,
}

impl Default for LossEstimator {
    fn default() -> Self {
        Self::new(LossEstimatorConfig::default())
    }
}

impl LossEstimator {
    /// Create with the given config.
    #[must_use]
    pub fn new(cfg: LossEstimatorConfig) -> Self {
        let alpha_ppt = cfg.alpha_ppt.clamp(1, 1000);
        Self {
            cfg: LossEstimatorConfig { alpha_ppt },
            loss_ppm: 0,
            initialized: false,
        }
    }

    /// Current loss estimate (ppm).
    #[must_use]
    pub fn loss_ppm(&self) -> u32 {
        self.loss_ppm
    }

    /// Observe an interval: `expected` packets, `received` of them arrived.
    ///
    /// # Errors
    /// Returns [`vd_common::Error::InvalidArgument`] if `received > expected`.
    pub fn push_interval(&mut self, expected: u32, received: u32) -> Result<u32, vd_common::Error> {
        if received > expected {
            return Err(vd_common::Error::InvalidArgument(format!(
                "received {received} > expected {expected}"
            )));
        }
        if expected == 0 {
            return Ok(self.loss_ppm);
        }
        let lost = expected - received;
        // ppm = lost/expected * 1_000_000
        let instant =
            u32::try_from(u64::from(lost).saturating_mul(1_000_000) / u64::from(expected))
                .unwrap_or(u32::MAX);

        if !self.initialized {
            self.loss_ppm = instant;
            self.initialized = true;
            return Ok(self.loss_ppm);
        }

        let a = u64::from(self.cfg.alpha_ppt);
        let inv = 1000 - a;
        let blended = (a * u64::from(instant) + inv * u64::from(self.loss_ppm)) / 1000;
        self.loss_ppm = u32::try_from(blended).unwrap_or(u32::MAX);
        Ok(self.loss_ppm)
    }

    /// Direct observation of an instant loss_ppm (e.g. from a ReceiverReport).
    pub fn push_ppm(&mut self, instant_ppm: u32) -> u32 {
        if !self.initialized {
            self.loss_ppm = instant_ppm;
            self.initialized = true;
            return self.loss_ppm;
        }
        let a = u64::from(self.cfg.alpha_ppt);
        let inv = 1000 - a;
        let blended = (a * u64::from(instant_ppm) + inv * u64::from(self.loss_ppm)) / 1000;
        self.loss_ppm = u32::try_from(blended).unwrap_or(u32::MAX);
        self.loss_ppm
    }
}

/// Delay-gradient + loss combined into one [`NetworkEstimate`].
#[derive(Debug, Clone)]
pub struct NetworkEstimator {
    delay: DelayGradientEstimator,
    loss: LossEstimator,
}

impl Default for NetworkEstimator {
    fn default() -> Self {
        Self::new(
            DelayGradientConfig::default(),
            LossEstimatorConfig::default(),
        )
    }
}

impl NetworkEstimator {
    /// Create with explicit sub-configs.
    #[must_use]
    pub fn new(delay: DelayGradientConfig, loss: LossEstimatorConfig) -> Self {
        Self {
            delay: DelayGradientEstimator::new(delay),
            loss: LossEstimator::new(loss),
        }
    }

    /// Access the delay trendline.
    #[must_use]
    pub fn delay(&self) -> &DelayGradientEstimator {
        &self.delay
    }

    /// Access the loss EWMA.
    #[must_use]
    pub fn loss(&self) -> &LossEstimator {
        &self.loss
    }

    /// Push an OWD sample and optional interval loss; returns the combined estimate.
    ///
    /// When `loss_interval` is `None`, the previous loss estimate is kept.
    ///
    /// # Errors
    /// Propagates loss interval validation errors.
    pub fn push(
        &mut self,
        t_ms: u64,
        owd_us: i64,
        loss_interval: Option<(u32, u32)>,
    ) -> Result<NetworkEstimate, vd_common::Error> {
        self.delay.push(t_ms, owd_us);
        if let Some((expected, received)) = loss_interval {
            self.loss.push_interval(expected, received)?;
        }
        Ok(self.estimate())
    }

    /// Update from a ready-made receiver report (smooths loss; feeds overuse detector).
    pub fn observe_report(&mut self, owd_gradient_us_per_s: i32, loss_ppm: u32) -> NetworkEstimate {
        self.delay.observe_gradient(owd_gradient_us_per_s);
        self.loss.push_ppm(loss_ppm);
        self.estimate()
    }

    /// Snapshot without mutating.
    #[must_use]
    pub fn estimate(&self) -> NetworkEstimate {
        NetworkEstimate {
            owd_gradient_us_per_s: self.delay.gradient_us_per_s(),
            loss_ppm: self.loss.loss_ppm(),
            usage: self.delay.usage(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_owd_trace_is_normal() {
        let mut e = DelayGradientEstimator::new(DelayGradientConfig {
            overuse_count: 1,
            underuse_count: 1,
            ..DelayGradientConfig::default()
        });
        for i in 0..15 {
            e.push(i * 50, 20_000); // constant 20 ms OWD
        }
        assert!(
            e.gradient_us_per_s().abs() < 500,
            "g={}",
            e.gradient_us_per_s()
        );
        assert_eq!(e.usage(), BandwidthUsage::Normal);
    }

    #[test]
    fn rising_owd_trace_overuses() {
        let mut e = DelayGradientEstimator::new(DelayGradientConfig {
            deadband_us_per_s: 500,
            overuse_count: 2,
            underuse_count: 2,
            window: 12,
        });
        // OWD grows 2 ms every 50 ms → 40_000 µs/s
        for i in 0..12 {
            e.push(i * 50, 10_000 + (i as i64) * 2_000);
        }
        assert!(
            e.gradient_us_per_s() > 20_000,
            "expected strong positive gradient, got {}",
            e.gradient_us_per_s()
        );
        assert_eq!(e.usage(), BandwidthUsage::Overuse);
    }

    #[test]
    fn falling_owd_trace_underuses() {
        let mut e = DelayGradientEstimator::new(DelayGradientConfig {
            deadband_us_per_s: 500,
            overuse_count: 2,
            underuse_count: 2,
            window: 12,
        });
        for i in 0..12 {
            e.push(i * 50, 40_000 - (i as i64) * 2_000);
        }
        assert!(
            e.gradient_us_per_s() < -20_000,
            "expected strong negative gradient, got {}",
            e.gradient_us_per_s()
        );
        assert_eq!(e.usage(), BandwidthUsage::Underuse);
    }

    #[test]
    fn loss_ewma_tracks_spike_then_recovers() {
        let mut loss = LossEstimator::new(LossEstimatorConfig { alpha_ppt: 500 });
        // 0% then 10% then 0%
        assert_eq!(loss.push_interval(100, 100).unwrap(), 0);
        let mid = loss.push_interval(100, 90).unwrap();
        assert!(mid > 40_000 && mid < 60_000, "mid={mid}");
        let later = loss.push_interval(100, 100).unwrap();
        assert!(later < mid, "should decay after clear interval");
    }

    #[test]
    fn network_estimator_synthetic_congestion_trace() {
        let mut net = NetworkEstimator::default();
        let mut last = NetworkEstimate {
            owd_gradient_us_per_s: 0,
            loss_ppm: 0,
            usage: BandwidthUsage::Normal,
        };
        // Healthy then congestion: rising delay + 5% loss
        for i in 0..8 {
            last = net.push(i * 50, 15_000, Some((100, 100))).expect("push");
        }
        assert_eq!(last.usage, BandwidthUsage::Normal);

        for i in 8..20 {
            let owd = 15_000 + ((i - 8) as i64) * 3_000;
            last = net.push(i * 50, owd, Some((100, 95))).expect("push");
        }
        assert_eq!(last.usage, BandwidthUsage::Overuse);
        assert!(last.loss_ppm > 10_000, "loss_ppm={}", last.loss_ppm);
        assert!(last.owd_gradient_us_per_s > 5_000);
    }
}
