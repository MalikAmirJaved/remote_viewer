//! Quality ladder levels and modes.

use serde::{Deserialize, Serialize};

/// Prefer resolution (text) or frame rate (motion).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QualityMode {
    /// Prefer resolution / sharpness for static/text desktops.
    #[default]
    TextSharp,
    /// Prefer fps for video/games.
    Motion,
}

/// One rung on the adaptive quality ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityLevel {
    /// Display width.
    pub width: u16,
    /// Display height.
    pub height: u16,
    /// Target frames per second.
    pub fps: u8,
    /// Nominal bitrate ceiling for this rung (bps).
    pub target_bps: u32,
}

/// Ordered ladder from highest to lowest quality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityLadder {
    levels: Vec<QualityLevel>,
}

impl Default for QualityLadder {
    fn default() -> Self {
        Self::standard()
    }
}

impl QualityLadder {
    /// Default ladder from docs (initial target, to be validated):
    /// 1080p@30 → 1080p@20 → 720p@30 → 720p@15 → 540p@15 → 360p@10
    #[must_use]
    pub fn standard() -> Self {
        Self {
            levels: vec![
                QualityLevel {
                    width: 1920,
                    height: 1080,
                    fps: 30,
                    target_bps: 4_000_000,
                },
                QualityLevel {
                    width: 1920,
                    height: 1080,
                    fps: 20,
                    target_bps: 2_500_000,
                },
                QualityLevel {
                    width: 1280,
                    height: 720,
                    fps: 30,
                    target_bps: 1_500_000,
                },
                QualityLevel {
                    width: 1280,
                    height: 720,
                    fps: 15,
                    target_bps: 800_000,
                },
                QualityLevel {
                    width: 960,
                    height: 540,
                    fps: 15,
                    target_bps: 500_000,
                },
                QualityLevel {
                    width: 640,
                    height: 360,
                    fps: 10,
                    target_bps: 250_000,
                },
            ],
        }
    }

    /// Number of levels.
    #[must_use]
    pub fn len(&self) -> usize {
        self.levels.len()
    }

    /// Empty check.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    /// Get level by index (0 = highest quality).
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&QualityLevel> {
        self.levels.get(index)
    }

    /// Clamp index into range.
    #[must_use]
    pub fn clamp_index(&self, index: usize) -> usize {
        index.min(self.levels.len().saturating_sub(1))
    }
}
