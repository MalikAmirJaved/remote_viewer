//! Shared error types. Libraries use `thiserror`; binaries may wrap with `anyhow`.

use thiserror::Error;

/// Result alias for remote_viewer libraries.
pub type Result<T> = std::result::Result<T, Error>;

/// Common error taxonomy used across crates.
#[derive(Debug, Error)]
pub enum Error {
    /// Feature or backend not yet implemented (P0 stubs return this — never `todo!()`).
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),

    /// Invalid argument or configuration.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Protocol / framing error.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// Cryptographic failure (handshake, verify, decrypt).
    #[error("crypto error: {0}")]
    Crypto(String),

    /// I/O or transport failure.
    #[error("io error: {0}")]
    Io(String),

    /// Resource unavailable (capture lost, encoder busy, etc.).
    #[error("unavailable: {0}")]
    Unavailable(String),

    /// Operation timed out.
    #[error("timeout: {0}")]
    Timeout(String),

    /// Consent denied or revoked by host user.
    #[error("consent denied")]
    ConsentDenied,

    /// Other wrapped error message.
    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
