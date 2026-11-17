//! Input injection: mouse/keyboard into the host OS.
//!
//! Unsafe allowed for FFI; each `unsafe` needs `// SAFETY:`.

use vd_common::{Error, Result};
use vd_proto::{InputMessage, InputMoveMessage};

/// Injects remote input into the local OS session.
pub trait InputInjector: Send {
    /// Inject a reliable input event (key/button/wheel/text/CAD).
    ///
    /// # Errors
    /// Platform injection failures.
    fn inject(&mut self, event: &InputMessage) -> Result<()>;

    /// Inject a mouse move (absolute or relative).
    ///
    /// # Errors
    /// Platform injection failures.
    fn inject_move(&mut self, event: &InputMoveMessage) -> Result<()>;

    /// Release all stuck keys/buttons (call on disconnect).
    ///
    /// # Errors
    /// Platform failures.
    fn release_all(&mut self) -> Result<()>;
}

/// Stub injector for all platforms.
#[derive(Debug, Default)]
pub struct StubInjector;

impl InputInjector for StubInjector {
    fn inject(&mut self, _event: &InputMessage) -> Result<()> {
        Err(Error::NotImplemented("InputInjector::inject"))
    }

    fn inject_move(&mut self, _event: &InputMoveMessage) -> Result<()> {
        Err(Error::NotImplemented("InputInjector::inject_move"))
    }

    fn release_all(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(windows)]
pub mod windows {
    //! Win32 `SendInput` backend (stub).
    use super::InputInjector;
    use vd_common::{Error, Result};
    use vd_proto::{InputMessage, InputMoveMessage};

    /// Windows SendInput injector stub.
    #[derive(Debug, Default)]
    pub struct SendInputInjector;

    impl InputInjector for SendInputInjector {
        fn inject(&mut self, _event: &InputMessage) -> Result<()> {
            Err(Error::NotImplemented("SendInput inject"))
        }

        fn inject_move(&mut self, _event: &InputMoveMessage) -> Result<()> {
            Err(Error::NotImplemented("SendInput inject_move"))
        }

        fn release_all(&mut self) -> Result<()> {
            Ok(())
        }
    }
}
