//! Screen capture abstraction. Platform modules are cfg-gated stubs for P0.
//!
//! Unsafe is allowed in platform backends; each `unsafe` block needs `// SAFETY:`.

pub mod fake;

use vd_common::{Error, Result};

pub use fake::{render_test_pattern, FakeCaptureConfig, FakeCapturer};

/// Pixel format of a captured frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 8-bit BGRA.
    Bgra8,
    /// NV12 (typical HW encoder input).
    Nv12,
}

/// A captured desktop frame (CPU copy in stubs; GPU textures later).
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel layout.
    pub format: PixelFormat,
    /// Capture timestamp (host monotonic µs).
    pub capture_ts_us: u64,
    /// Dirty rectangles (empty = full frame).
    pub dirty_rects: Vec<(u32, u32, u32, u32)>,
    /// Raw bytes (BGRA/NV12). Empty in stub.
    pub data: Vec<u8>,
}

/// Screen capturer trait.
pub trait Capturer: Send {
    /// Begin capturing the selected monitor.
    ///
    /// # Errors
    /// Platform capture failures.
    fn start(&mut self, monitor_index: u32) -> Result<()>;

    /// Grab the next frame if available (non-blocking preferred).
    ///
    /// Returns `Ok(None)` when nothing changed (dirty-rect skip).
    ///
    /// # Errors
    /// Access-lost and other capture failures.
    fn capture_frame(&mut self) -> Result<Option<CapturedFrame>>;

    /// Stop capturing.
    ///
    /// # Errors
    /// Platform failures on stop.
    fn stop(&mut self) -> Result<()>;
}

/// Stub capturer for compile/link on all platforms.
#[derive(Debug, Default)]
pub struct StubCapturer;

impl Capturer for StubCapturer {
    fn start(&mut self, _monitor_index: u32) -> Result<()> {
        Err(Error::NotImplemented("Capturer::start"))
    }

    fn capture_frame(&mut self) -> Result<Option<CapturedFrame>> {
        Err(Error::NotImplemented("Capturer::capture_frame"))
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(windows)]
pub mod windows {
    //! DXGI Desktop Duplication / WGC backends (stubs).
    use super::{CapturedFrame, Capturer};
    use vd_common::{Error, Result};

    /// Windows DXGI Desktop Duplication capturer (stub).
    #[derive(Debug, Default)]
    pub struct DxgiCapturer;

    impl Capturer for DxgiCapturer {
        fn start(&mut self, _monitor_index: u32) -> Result<()> {
            Err(Error::NotImplemented("DXGI Desktop Duplication"))
        }

        fn capture_frame(&mut self) -> Result<Option<CapturedFrame>> {
            Err(Error::NotImplemented("DXGI capture_frame"))
        }

        fn stop(&mut self) -> Result<()> {
            Ok(())
        }
    }
}

#[cfg(target_os = "linux")]
pub mod linux {
    //! PipeWire / X11 capturers (stubs).
    use super::{CapturedFrame, Capturer};
    use vd_common::{Error, Result};

    /// PipeWire capturer stub.
    #[derive(Debug, Default)]
    pub struct PipeWireCapturer;

    impl Capturer for PipeWireCapturer {
        fn start(&mut self, _monitor_index: u32) -> Result<()> {
            Err(Error::NotImplemented("PipeWire capturer"))
        }

        fn capture_frame(&mut self) -> Result<Option<CapturedFrame>> {
            Err(Error::NotImplemented("PipeWire capture_frame"))
        }

        fn stop(&mut self) -> Result<()> {
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
pub mod macos {
    //! ScreenCaptureKit capturer (stub).
    use super::{CapturedFrame, Capturer};
    use vd_common::{Error, Result};

    /// ScreenCaptureKit capturer stub.
    #[derive(Debug, Default)]
    pub struct SckCapturer;

    impl Capturer for SckCapturer {
        fn start(&mut self, _monitor_index: u32) -> Result<()> {
            Err(Error::NotImplemented("ScreenCaptureKit"))
        }

        fn capture_frame(&mut self) -> Result<Option<CapturedFrame>> {
            Err(Error::NotImplemented("SCK capture_frame"))
        }

        fn stop(&mut self) -> Result<()> {
            Ok(())
        }
    }
}
