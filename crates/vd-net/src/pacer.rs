//! Token-bucket video pacer (T-0207).
//!
//! Smooths datagram sends so a burst of fragments cannot overflow a weak-link
//! buffer. The fill rate follows the `vd-rate` target bitrate; the burst capacity
//! defaults to ~50 ms of that rate (at least one max fragment).
//!
//! Time is injected as monotonic µs (`now_us`) so unit tests can advance a virtual
//! clock without sleeping. Production code should pass `MonotonicClock::now_us().0`.

use crate::video::DEFAULT_MAX_FRAGMENT_DATA;
use vd_common::{Error, Result};

/// Default burst window used by [`Pacer::new`] (milliseconds of target bitrate).
pub const DEFAULT_BURST_MS: u32 = 50;

/// Relative tolerance used by the rate-enforcement unit test (±).
pub const RATE_TEST_TOLERANCE: f64 = 0.10;

/// Token-bucket pacer: spend bytes when tokens are available.
#[derive(Debug, Clone)]
pub struct Pacer {
    bitrate_bps: u32,
    capacity_bytes: f64,
    tokens: f64,
    last_us: Option<u64>,
}

impl Pacer {
    /// Create a pacer at `bitrate_bps` with a [`DEFAULT_BURST_MS`] burst.
    ///
    /// # Errors
    /// [`Error::InvalidArgument`] when `bitrate_bps` is zero.
    pub fn new(bitrate_bps: u32) -> Result<Self> {
        Self::with_burst_ms(bitrate_bps, DEFAULT_BURST_MS)
    }

    /// Create a pacer with an explicit burst window in milliseconds.
    ///
    /// Capacity is `max(bitrate_bps/8 * burst_ms/1000, DEFAULT_MAX_FRAGMENT_DATA)` so a
    /// single video fragment can always clear an empty bucket at the configured rate.
    ///
    /// # Errors
    /// Zero bitrate or zero burst window.
    pub fn with_burst_ms(bitrate_bps: u32, burst_ms: u32) -> Result<Self> {
        if bitrate_bps == 0 {
            return Err(Error::InvalidArgument(
                "pacer: bitrate_bps must be non-zero".into(),
            ));
        }
        if burst_ms == 0 {
            return Err(Error::InvalidArgument(
                "pacer: burst_ms must be non-zero".into(),
            ));
        }
        let from_rate = f64::from(bitrate_bps) / 8.0 * f64::from(burst_ms) / 1000.0;
        let capacity = from_rate.max(DEFAULT_MAX_FRAGMENT_DATA as f64);
        Ok(Self {
            bitrate_bps,
            capacity_bytes: capacity,
            tokens: capacity,
            last_us: None,
        })
    }

    /// Create a pacer with an explicit capacity in bytes (tests / special paths).
    ///
    /// # Errors
    /// Zero bitrate or zero capacity.
    pub fn with_capacity_bytes(bitrate_bps: u32, capacity_bytes: u64) -> Result<Self> {
        if bitrate_bps == 0 {
            return Err(Error::InvalidArgument(
                "pacer: bitrate_bps must be non-zero".into(),
            ));
        }
        if capacity_bytes == 0 {
            return Err(Error::InvalidArgument(
                "pacer: capacity_bytes must be non-zero".into(),
            ));
        }
        let capacity = capacity_bytes as f64;
        Ok(Self {
            bitrate_bps,
            capacity_bytes: capacity,
            tokens: capacity,
            last_us: None,
        })
    }

    /// Current target bitrate.
    #[must_use]
    pub fn bitrate_bps(&self) -> u32 {
        self.bitrate_bps
    }

    /// Burst capacity in bytes.
    #[must_use]
    pub fn capacity_bytes(&self) -> u64 {
        self.capacity_bytes as u64
    }

    /// Update the target bitrate (e.g. from `vd-rate`). Keeps current tokens, clamped
    /// to the new capacity derived from [`DEFAULT_BURST_MS`].
    ///
    /// # Errors
    /// Zero bitrate.
    pub fn set_bitrate(&mut self, bitrate_bps: u32) -> Result<()> {
        if bitrate_bps == 0 {
            return Err(Error::InvalidArgument(
                "pacer: bitrate_bps must be non-zero".into(),
            ));
        }
        self.bitrate_bps = bitrate_bps;
        let from_rate = f64::from(bitrate_bps) / 8.0 * f64::from(DEFAULT_BURST_MS) / 1000.0;
        self.capacity_bytes = from_rate.max(DEFAULT_MAX_FRAGMENT_DATA as f64);
        self.tokens = self.tokens.min(self.capacity_bytes);
        Ok(())
    }

    /// Refill from `now_us` and spend `size` bytes if possible.
    ///
    /// Returns `true` when the packet may go on the wire. A zero-size packet is always
    /// allowed and does not spend tokens. Time going backwards is treated as "no
    /// refill" (clock skew / test rewind).
    #[must_use]
    pub fn allow(&mut self, size: usize, now_us: u64) -> bool {
        self.refill(now_us);
        if size == 0 {
            return true;
        }
        let need = size as f64;
        if self.tokens + f64::EPSILON < need {
            return false;
        }
        self.tokens -= need;
        true
    }

    /// Microseconds to wait until `size` would be allowed at the current rate, or `0`
    /// if it is already allowed. Does not mutate the bucket.
    #[must_use]
    pub fn wait_us(&self, size: usize, now_us: u64) -> u64 {
        if size == 0 {
            return 0;
        }
        let tokens = self.tokens_at(now_us);
        let need = size as f64;
        if tokens + f64::EPSILON >= need {
            return 0;
        }
        let missing = need - tokens;
        let bytes_per_us = f64::from(self.bitrate_bps) / 8.0 / 1_000_000.0;
        if bytes_per_us <= 0.0 {
            return u64::MAX;
        }
        (missing / bytes_per_us).ceil() as u64
    }

    fn refill(&mut self, now_us: u64) {
        let Some(last) = self.last_us else {
            self.last_us = Some(now_us);
            return;
        };
        if now_us <= last {
            self.last_us = Some(now_us);
            return;
        }
        let elapsed = (now_us - last) as f64;
        let bytes_per_us = f64::from(self.bitrate_bps) / 8.0 / 1_000_000.0;
        self.tokens = (self.tokens + elapsed * bytes_per_us).min(self.capacity_bytes);
        self.last_us = Some(now_us);
    }

    fn tokens_at(&self, now_us: u64) -> f64 {
        let Some(last) = self.last_us else {
            return self.tokens;
        };
        if now_us <= last {
            return self.tokens;
        }
        let elapsed = (now_us - last) as f64;
        let bytes_per_us = f64::from(self.bitrate_bps) / 8.0 / 1_000_000.0;
        (self.tokens + elapsed * bytes_per_us).min(self.capacity_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::{Pacer, DEFAULT_BURST_MS, RATE_TEST_TOLERANCE};
    use crate::video::DEFAULT_MAX_FRAGMENT_DATA;
    use vd_common::Error;

    #[test]
    fn zero_bitrate_is_rejected() {
        assert!(matches!(Pacer::new(0), Err(Error::InvalidArgument(_))));
        assert!(matches!(
            Pacer::with_capacity_bytes(1_000_000, 0),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn burst_allows_an_immediate_fragment_then_blocks() {
        // Tiny rate, capacity exactly one 1150 B fragment.
        let mut p =
            Pacer::with_capacity_bytes(8_000, DEFAULT_MAX_FRAGMENT_DATA as u64).expect("pacer");
        assert!(p.allow(DEFAULT_MAX_FRAGMENT_DATA, 0));
        assert!(!p.allow(1, 0), "bucket empty: must block");
        // After enough time for 1 byte at 8000 bps = 1000 B/s → 1 ms per byte.
        assert!(p.allow(1, 1_000));
    }

    #[test]
    fn enforces_target_bps_within_tolerance() {
        // 800 kbps = 100_000 bytes/s. Over 200 ms ideal = 20_000 bytes.
        const BITRATE: u32 = 800_000;
        const PACKET: usize = 500;
        const DURATION_US: u64 = 200_000;

        let mut p = Pacer::with_capacity_bytes(BITRATE, PACKET as u64).expect("pacer");
        let mut now = 0u64;
        let mut sent = 0u64;
        // Start empty-ish after draining the initial burst capacity.
        assert!(p.allow(PACKET, now));
        sent += PACKET as u64;

        while now < DURATION_US {
            let wait = p.wait_us(PACKET, now);
            now = now.saturating_add(wait.max(1));
            if now > DURATION_US {
                break;
            }
            if p.allow(PACKET, now) {
                sent += PACKET as u64;
            }
        }

        let ideal = f64::from(BITRATE) / 8.0 * (DURATION_US as f64 / 1_000_000.0);
        let ratio = sent as f64 / ideal;
        assert!(
            (1.0 - RATE_TEST_TOLERANCE..=1.0 + RATE_TEST_TOLERANCE).contains(&ratio),
            "sent {sent} B over {DURATION_US} µs at {BITRATE} bps; ideal {ideal:.0}; ratio {ratio:.3} outside ±{RATE_TEST_TOLERANCE}"
        );
    }

    #[test]
    fn set_bitrate_updates_capacity_and_rate() {
        let mut p = Pacer::new(1_000_000).expect("pacer");
        assert_eq!(p.bitrate_bps(), 1_000_000);
        let before = p.capacity_bytes();
        p.set_bitrate(4_000_000).expect("set");
        assert_eq!(p.bitrate_bps(), 4_000_000);
        assert!(p.capacity_bytes() >= before);
        // 50 ms of 4 Mbps = 25_000 B
        let expected = (4_000_000u64 / 8) * u64::from(DEFAULT_BURST_MS) / 1000;
        assert_eq!(
            p.capacity_bytes(),
            expected.max(DEFAULT_MAX_FRAGMENT_DATA as u64)
        );
    }

    #[test]
    fn wait_us_is_zero_when_tokens_cover_the_packet() {
        let p = Pacer::with_capacity_bytes(1_000_000, 2000).expect("pacer");
        assert_eq!(p.wait_us(1000, 0), 0);
    }
}
