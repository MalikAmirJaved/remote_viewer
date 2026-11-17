//! Clipboard synchronization trait and platform stubs.
#![forbid(unsafe_code)]

use vd_common::{Error, Result};

/// Clipboard sync interface (consent-gated for large payloads).
pub trait ClipboardSync: Send {
    /// Read local clipboard as UTF-8 text if available.
    ///
    /// # Errors
    /// Platform clipboard failures.
    fn read_text(&mut self) -> Result<Option<String>>;

    /// Write UTF-8 text to the local clipboard.
    ///
    /// # Errors
    /// Platform clipboard failures.
    fn write_text(&mut self, text: &str) -> Result<()>;
}

/// Stub clipboard.
#[derive(Debug, Default)]
pub struct StubClipboard;

impl ClipboardSync for StubClipboard {
    fn read_text(&mut self) -> Result<Option<String>> {
        Err(Error::NotImplemented("ClipboardSync::read_text"))
    }

    fn write_text(&mut self, _text: &str) -> Result<()> {
        Err(Error::NotImplemented("ClipboardSync::write_text"))
    }
}
