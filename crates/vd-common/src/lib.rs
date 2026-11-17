//! Shared primitives for remote_viewer: IDs, monotonic clock, errors, config, tracing init.
#![forbid(unsafe_code)]

pub mod clock;
pub mod config;
pub mod error;
pub mod id;
pub mod telemetry;

pub use clock::{InstantUs, MonotonicClock};
pub use config::CommonConfig;
pub use error::{Error, Result};
pub use id::{DeviceId, PeerId, SessionId};
pub use telemetry::init_tracing;
