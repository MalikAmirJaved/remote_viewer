//! Tracing subscriber initialization for binaries and integration tests.

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize a default `tracing` subscriber from `RUST_LOG` (default: `info`).
///
/// Safe to call once per process. Subsequent calls are ignored if a global
/// subscriber is already set.
pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .try_init();
}
