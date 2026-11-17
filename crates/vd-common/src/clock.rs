//! Monotonic microsecond clock for latency telemetry and protocol timestamps.

use std::time::Instant;

/// Instant expressed as microseconds since an arbitrary process-local epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstantUs(pub u64);

impl InstantUs {
    /// Saturating duration since `earlier`, in microseconds.
    #[must_use]
    pub fn saturating_duration_since(self, earlier: Self) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
}

/// Process-local monotonic clock with microsecond resolution.
///
/// Epoch is the first call to [`MonotonicClock::new`] (or first `now` if constructed
/// via [`Default`]). Values are **not** comparable across processes — use for
/// relative latency only. Cross-host sync is a separate concern (see performance budget).
#[derive(Debug, Clone)]
pub struct MonotonicClock {
    origin: Instant,
}

impl Default for MonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MonotonicClock {
    /// Create a clock whose epoch is "now".
    #[must_use]
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }

    /// Current time as microseconds since this clock's epoch.
    #[must_use]
    pub fn now_us(&self) -> InstantUs {
        let elapsed = self.origin.elapsed();
        let us = u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX);
        InstantUs(us)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn clock_is_monotonic() {
        let clock = MonotonicClock::new();
        let a = clock.now_us();
        thread::sleep(Duration::from_millis(2));
        let b = clock.now_us();
        assert!(b.0 >= a.0);
        assert!(b.saturating_duration_since(a) >= 1_000);
    }

    #[test]
    fn saturating_duration_handles_inversion() {
        let later = InstantUs(100);
        let earlier = InstantUs(200);
        assert_eq!(later.saturating_duration_since(earlier), 0);
    }
}
