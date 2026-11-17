//! In-memory deterministic lossy / latency / bandwidth-limited link for tests.
//!
//! Used by rate-control and transport tests without real sockets.
#![forbid(unsafe_code)]

use bytes::Bytes;
use std::collections::VecDeque;
use vd_common::{Error, Result};

/// Link impairment profile.
#[derive(Debug, Clone, Default)]
pub struct LinkConfig {
    /// One-way base latency in milliseconds.
    pub latency_ms: u32,
    /// Additional jitter in milliseconds (uniform 0..=jitter applied per packet).
    pub jitter_ms: u32,
    /// Independent packet loss percent (0–100).
    pub loss_pct: u32,
    /// Token-bucket bandwidth limit in kilobits per second (`0` = unlimited).
    pub bandwidth_kbps: u32,
    /// Probability percent that two consecutive deliveries swap order (0–100).
    pub reorder_pct: u32,
}

impl LinkConfig {
    /// LAN ideal profile.
    #[must_use]
    pub fn lan() -> Self {
        Self::default()
    }

    /// Weak WAN profile matching docs/05_PERFORMANCE_BUDGET.md.
    #[must_use]
    pub fn wan_weak() -> Self {
        Self {
            latency_ms: 75, // one-way ≈ RTT/2 for 150 ms RTT
            jitter_ms: 10,
            loss_pct: 3,
            bandwidth_kbps: 1000,
            reorder_pct: 1,
        }
    }
}

#[derive(Debug, Clone)]
struct ScheduledPacket {
    deliver_at_ms: u64,
    payload: Bytes,
}

/// Deterministic in-process unidirectional link.
///
/// Uses an injected RNG seed for reproducibility. Time advances via [`SimLink::advance_ms`].
#[derive(Debug)]
pub struct SimLink {
    cfg: LinkConfig,
    /// Simple LCG state for deterministic "randomness".
    rng: u64,
    now_ms: u64,
    queue: VecDeque<ScheduledPacket>,
    /// Bytes that may be sent this millisecond window (token bucket).
    tokens: f64,
    last_reorder_candidate: Option<ScheduledPacket>,
}

impl SimLink {
    /// Create a link with the given config and RNG seed.
    #[must_use]
    pub fn new(cfg: LinkConfig, seed: u64) -> Self {
        Self {
            cfg,
            rng: seed | 1, // avoid zero state
            now_ms: 0,
            queue: VecDeque::new(),
            tokens: 0.0,
            last_reorder_candidate: None,
        }
    }

    /// Current simulated time in milliseconds.
    #[must_use]
    pub fn now_ms(&self) -> u64 {
        self.now_ms
    }

    fn next_u32(&mut self) -> u32 {
        // Numerical Recipes LCG
        self.rng = self
            .rng
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (self.rng >> 32) as u32
    }

    fn roll_pct(&mut self, pct: u32) -> bool {
        if pct == 0 {
            return false;
        }
        if pct >= 100 {
            return true;
        }
        (self.next_u32() % 100) < pct
    }

    /// Enqueue a packet for eventual delivery (subject to loss/bw/reorder).
    pub fn send(&mut self, payload: impl Into<Bytes>) -> Result<()> {
        let payload = payload.into();
        if self.roll_pct(self.cfg.loss_pct) {
            return Ok(()); // dropped
        }

        // Bandwidth: require tokens; if limited and empty, reject (caller may retry).
        if self.cfg.bandwidth_kbps > 0 {
            let need = payload.len() as f64;
            if self.tokens < need {
                return Err(Error::Unavailable(
                    "simlink bandwidth tokens exhausted".into(),
                ));
            }
            self.tokens -= need;
        }

        let jitter = if self.cfg.jitter_ms == 0 {
            0
        } else {
            self.next_u32() % (self.cfg.jitter_ms + 1)
        };
        let deliver_at_ms = self.now_ms + u64::from(self.cfg.latency_ms + jitter);

        let pkt = ScheduledPacket {
            deliver_at_ms,
            payload,
        };

        if self.roll_pct(self.cfg.reorder_pct) {
            if let Some(prev) = self.last_reorder_candidate.take() {
                // Swap delivery times to induce reorder.
                let mut a = prev;
                let mut b = pkt;
                std::mem::swap(&mut a.deliver_at_ms, &mut b.deliver_at_ms);
                self.queue.push_back(a);
                self.queue.push_back(b);
                return Ok(());
            }
            self.last_reorder_candidate = Some(pkt);
            return Ok(());
        }

        if let Some(pending) = self.last_reorder_candidate.take() {
            self.queue.push_back(pending);
        }
        self.queue.push_back(pkt);
        Ok(())
    }

    /// Advance simulated time and refill bandwidth tokens.
    pub fn advance_ms(&mut self, ms: u64) {
        self.now_ms = self.now_ms.saturating_add(ms);
        if self.cfg.bandwidth_kbps > 0 {
            // tokens in bytes: kbps * ms / 8
            let add = f64::from(self.cfg.bandwidth_kbps) * ms as f64 / 8.0;
            self.tokens = (self.tokens + add).min(f64::from(self.cfg.bandwidth_kbps) * 50.0);
        }
        if let Some(pending) = self.last_reorder_candidate.take() {
            self.queue.push_back(pending);
        }
    }

    /// Pop all packets whose delivery time has been reached (sorted by deliver_at).
    pub fn recv_ready(&mut self) -> Vec<Bytes> {
        let mut ready: Vec<ScheduledPacket> = self
            .queue
            .iter()
            .filter(|p| p.deliver_at_ms <= self.now_ms)
            .cloned()
            .collect();
        self.queue.retain(|p| p.deliver_at_ms > self.now_ms);
        ready.sort_by_key(|p| p.deliver_at_ms);
        ready.into_iter().map(|p| p.payload).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lossless_zero_latency_delivers_immediately() {
        let mut link = SimLink::new(LinkConfig::lan(), 42);
        link.send(&b"hello"[..]).unwrap();
        link.advance_ms(0);
        let got = link.recv_ready();
        assert_eq!(got.len(), 1);
        assert_eq!(&got[0][..], b"hello");
    }

    #[test]
    fn latency_defers_delivery() {
        let mut link = SimLink::new(
            LinkConfig {
                latency_ms: 50,
                ..LinkConfig::default()
            },
            7,
        );
        link.send(&b"x"[..]).unwrap();
        link.advance_ms(49);
        assert_eq!(link.recv_ready().len(), 0);
        link.advance_ms(1);
        assert_eq!(link.recv_ready().len(), 1);
    }

    #[test]
    fn total_loss_drops_all() {
        let mut link = SimLink::new(
            LinkConfig {
                loss_pct: 100,
                ..LinkConfig::default()
            },
            1,
        );
        for _ in 0..20 {
            link.send(&b"pkt"[..]).unwrap();
        }
        link.advance_ms(10);
        assert_eq!(link.recv_ready().len(), 0);
    }

    #[test]
    fn bandwidth_limit_rejects_when_empty() {
        let mut link = SimLink::new(
            LinkConfig {
                bandwidth_kbps: 8, // 1 byte/ms
                ..LinkConfig::default()
            },
            99,
        );
        // No tokens yet
        let err = link.send(vec![0u8; 64]);
        assert!(err.is_err());
        link.advance_ms(1000);
        assert!(link.send(vec![0u8; 64]).is_ok());
    }
}
