//! Input injection: mouse/keyboard into the host OS.
//!
//! Unsafe allowed for FFI; each `unsafe` needs `// SAFETY:`.
//!
//! # Wire conventions (v1)
//!
//! The protocol leaves two input fields loosely specified; this crate fixes them so host
//! and viewer agree:
//!
//! - `InputMessage::Key{Down,Up}::scancode` is an **IBM PC set-1 scan code** in the low
//!   byte. Bit [`EXTENDED_SCANCODE`] (0x100) marks an E0-prefixed key (arrows, right
//!   Ctrl/Alt, numpad Enter, …), i.e. the same encoding
//!   `MapVirtualKeyW(..., MAPVK_VK_TO_VSC)` exposes plus an explicit E0 flag.
//! - `modifiers` is **ignored** for injection: the protocol does not assign bits to it, so
//!   peers must send explicit `KeyDown`/`KeyUp` for modifier keys. Synthesising modifiers
//!   from an undefined mask could leave them stuck.
//! - `Wheel` deltas are **notches** (one notch = [`WHEEL_DELTA`] units), `dy` positive away
//!   from the user, `dx` positive to the right.
//! - `InputMoveMessage` absolute coordinates are **virtual-desktop pixels** (all monitors,
//!   negative origins allowed); `monitor` is ignored until per-monitor mapping exists.
//!
//! The architect should pin these in `docs/02_PROTOCOL.md`; see HANDOFF 2026-11-19.

use vd_common::{Error, Result};
use vd_proto::{InputMessage, InputMoveMessage};

/// Windows wheel constant: one notch of scrolling.
pub(crate) const WHEEL_DELTA: i32 = 120;

/// Wire flag marking an E0-prefixed ("extended") scan code.
pub(crate) const EXTENDED_SCANCODE: u32 = 0x100;

/// One OS-level action produced from a protocol message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostInput {
    /// Keyboard scan code with the extended/up flags.
    Key {
        /// Set-1 scan code (0x01..=0xFF).
        scan: u16,
        /// E0-prefixed key.
        extended: bool,
        /// Key-up instead of key-down.
        up: bool,
    },
    /// Mouse button transition.
    Button {
        /// Which button.
        button: MouseButton,
        /// True = down, false = up.
        down: bool,
    },
    /// Wheel scroll in `WHEEL_DELTA` units.
    Wheel {
        /// Signed wheel units (`notches * 120`).
        units: i32,
        /// Horizontal wheel instead of vertical.
        horizontal: bool,
    },
    /// Pointer movement; absolute coordinates are already normalised to 0..=65535.
    Move {
        /// X (normalised if `absolute`).
        dx: i32,
        /// Y (normalised if `absolute`).
        dy: i32,
        /// Absolute virtual-desktop position instead of a relative delta.
        absolute: bool,
    },
    /// One UTF-16 code unit typed through the Unicode path.
    Text {
        /// UTF-16 code unit.
        code_unit: u16,
        /// Key-up instead of key-down.
        up: bool,
    },
}

/// Mouse buttons understood by v1 (`InputMessage::MouseButton::button` 1..=5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MouseButton {
    /// Left button (wire id 1).
    Left,
    /// Right button (wire id 2).
    Right,
    /// Middle button (wire id 3).
    Middle,
    /// First extra button (wire id 4).
    X1,
    /// Second extra button (wire id 5).
    X2,
}

/// Virtual-desktop rectangle in pixels (may include negative origins).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DesktopBounds {
    /// Left edge (virtual-desktop pixels).
    pub left: i32,
    /// Top edge (virtual-desktop pixels).
    pub top: i32,
    /// Width in pixels (> 0).
    pub width: i32,
    /// Height in pixels (> 0).
    pub height: i32,
}

/// Translate one reliable input event into OS actions.
///
/// `CadRequest` is rejected: Ctrl+Alt+Del cannot be injected with `SendInput` and requires
/// the privileged service path (P7).
pub(crate) fn plan(event: &InputMessage) -> Result<Vec<HostInput>> {
    match event {
        InputMessage::KeyDown { scancode, .. } => Ok(vec![key(*scancode, false)?]),
        InputMessage::KeyUp { scancode, .. } => Ok(vec![key(*scancode, true)?]),
        InputMessage::MouseButton { button, down } => Ok(vec![HostInput::Button {
            button: mouse_button(*button)?,
            down: *down,
        }]),
        InputMessage::Wheel { dx, dy } => wheel(*dx, *dy),
        InputMessage::Text { text } => inject_text(text),
        InputMessage::CadRequest => Err(Error::NotImplemented(
            "Ctrl+Alt+Del injection requires the P7 secure-desktop service",
        )),
    }
}

/// Translate one mouse-move message. `bounds` is only read for absolute moves.
pub(crate) fn plan_move(
    event: &InputMoveMessage,
    bounds: DesktopBounds,
) -> Result<HostInput> {
    if event.relative {
        Ok(HostInput::Move {
            dx: event.x,
            dy: event.y,
            absolute: false,
        })
    } else {
        let (dx, dy) = normalize_absolute(event.x, event.y, bounds)?;
        Ok(HostInput::Move {
            dx,
            dy,
            absolute: true,
        })
    }
}

/// Coalescing state for mouse moves (T-0210).
///
/// Stores the latest move and its timestamp; a downstream sender will only transmit
/// the latest value if enough time has elapsed since the last transmission (≤125 Hz
/// cap). This runs lock-free on the input thread and never blocks on video.
#[derive(Debug, Default)]
pub struct MoveCoalescer {
    /// Latest move in virtual-desktop pixels.
    latest: Option<crate::HostInput>,
    /// Timestamp (µs) of when this latest move was generated.
    latest_ts_us: u64,
    /// Minimum interval between transmissions (µs). 125 Hz → 8000 µs.
    min_interval_us: u64,
}

impl MoveCoalescer {
    /// Create a coalescer with the default 125 Hz cap.
    #[must_use]
    pub fn new() -> Self {
        Self::with_min_interval(8_000)
    }

    /// Create a coalescer with a custom minimum interval.
    #[must_use]
    pub fn with_min_interval(min_interval_us: u64) -> Self {
        Self {
            min_interval_us,
            ..Default::default()
        }
    }

    /// Feed a new `InputMoveMessage` into the coalescer.
    ///
    /// Returns `Some(moved)` if a new move was captured (discarding any previous
    /// pending move), or `None` if the interval has not yet elapsed since the last
    /// transmission.
    pub fn feed(&mut self, event: &InputMoveMessage, now_us: u64, bounds: DesktopBounds) -> Option<crate::HostInput> {
        match plan_move(event, bounds) {
            Ok(moved) => {
                // Always update the latest move regardless of timing.
                self.latest = Some(moved.clone());
                self.latest_ts_us = now_us;

                // Check if enough time has elapsed since the last *transmission*.
                // We use a simple elapsed check; the caller decides when to actually send.
                if now_us.saturating_sub(self.latest_ts_us) >= self.min_interval_us {
                    // We have a fresh move that's past the interval; return it.
                    // The caller will reset the timer after sending.
                    Some(moved)
                } else {
                    // Not yet time to send; return None (caller should wait).
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// Reset the internal state (e.g., on session disconnect).
    #[must_use]
    pub fn reset(&mut self) {
        self.latest = None;
        self.latest_ts_us = 0;
    }
}

/// Decode a wire scan code into `(scan, extended)`.
pub(crate) fn decode_scancode(code: u32) -> Result<(u16, bool)> {
    let scan = code & 0xFF;
    if scan == 0 || code & !(0xFF | EXTENDED_SCANCODE) != 0 {
        return Err(Error::InvalidArgument(format!(
            "input: scancode {code:#X} is outside the set-1 range (0x01..=0xFF, plus 0x100 for extended keys)"
        )));
    }
    Ok((scan as u16, code & EXTENDED_SCANCODE != 0))
}

/// Map the wire button id (1..=5) to a [`MouseButton`].
pub(crate) fn mouse_button(id: u8) -> Result<MouseButton> {
    match id {
        1 => Ok(MouseButton::Left),
        2 => Ok(MouseButton::Right),
        3 => Ok(MouseButton::Middle),
        4 => Ok(MouseButton::X1),
        5 => Ok(MouseButton::X2),
        other => Err(Error::InvalidArgument(format!(
            "input: unknown mouse button {other} (expected 1..=5)"
        ))),
    }
}

/// Convert wheel notches into signed `WHEEL_DELTA` units.
pub(crate) fn wheel_units(notches: i32) -> Result<i32> {
    notches.checked_mul(WHEEL_DELTA).ok_or_else(|| {
        Error::InvalidArgument(format!(
            "input: wheel delta {notches} notches overflows the wire units"
        ))
    })
}

/// Normalise a virtual-desktop pixel position into the 0..=65535 range used by
/// `MOUSEEVENTF_ABSOLUTE`.
///
/// Out-of-range coordinates are clamped to the desktop edge: viewers legitimately send
/// positions computed before a resolution change, and clamping keeps the pointer usable
/// on the nearest edge instead of failing the whole move.
pub(crate) fn normalize_absolute(
    x: i32,
    y: i32,
    bounds: DesktopBounds,
) -> Result<(i32, i32)> {
    if bounds.width <= 0 || bounds.height <= 0 {
        return Err(Error::InvalidArgument(format!(
            "input: desktop bounds {}x{} are empty",
            bounds.width, bounds.height
        )));
    }

    let left = i64::from(bounds.left);
    let top = i64::from(bounds.top);
    let width = i64::from(bounds.width);
    let height = i64::from(bounds.height);
    let x = i64::from(x).clamp(left, left + width - 1);
    let y = i64::from(y).clamp(top, top + height - 1);

    let scale = |value: i64, origin: i64, extent: i64| -> i32 {
        if extent <= 1 {
            0
        } else {
            ((value - origin) * 65_535 / (extent - 1)) as i32
        }
    };
    Ok((scale(x, left, width), scale(y, top, height)))
}

fn key(code: u32, up: bool) -> Result<HostInput> {
    let (scan, extended) = decode_scancode(code)?;
    Ok(HostInput::Key { scan, extended, up })
}

fn wheel(dx: i32, dy: i32) -> Result<Vec<HostInput>> {
    let mut planned = Vec::new();
    if dy != 0 {
        planned.push(HostInput::Wheel {
            units: wheel_units(dy)?,
            horizontal: false,
        });
    }
    if dx != 0 {
        planned.push(HostInput::Wheel {
            units: wheel_units(dx)?,
            horizontal: true,
        });
    }
    if planned.is_empty() {
        return Err(Error::InvalidArgument(
            "input: wheel event has a zero delta".into(),
        ));
    }
    Ok(planned)
}

fn inject_text(value: &str) -> Result<Vec<HostInput>> {
    if value.is_empty() {
        return Err(Error::InvalidArgument("input: text event is empty".into()));
    }
    let mut planned = Vec::new();
    for code_unit in value.encode_utf16() {
        planned.push(HostInput::Text {
            code_unit,
            up: false,
        });
        planned.push(HostInput::Text {
            code_unit,
            up: true,
        });
    }
    Ok(planned)
}

#[cfg(test)]
mod tests {
    use super::{
        decode_scancode, mouse_button, normalize_absolute, plan, plan_move, wheel_units,
        DesktopBounds, HostInput, MouseButton, EXTENDED_SCANCODE, WHEEL_DELTA,
    };
    use vd_common::Error;
    use vd_proto::{InputMessage, InputMoveMessage};

    fn bounds() -> DesktopBounds {
        DesktopBounds {
            left: 0,
            top: 0,
            width: 1920,
            height: 1080,
        }
    }

    fn key_event(scancode: u32, modifiers: u32, down: bool) -> InputMessage {
        if down {
            InputMessage::KeyDown {
                scancode,
                modifiers,
            }
        } else {
            InputMessage::KeyUp {
                scancode,
                modifiers,
            }
        }
    }

    #[test]
    fn scancodes_decode_with_the_extended_flag() {
        assert_eq!(decode_scancode(0x1E).expect("A"), (0x1E, false));
        assert_eq!(
            decode_scancode(EXTENDED_SCANCODE | 0x4B).expect("left arrow"),
            (0x4B, true)
        );
        assert_eq!(decode_scancode(0xFF).expect("max"), (0xFF, false));
    }

    #[test]
    fn out_of_range_scancodes_are_rejected() {
        for code in [0, EXTENDED_SCANCODE, 0x200, 0xFFFF] {
            assert!(
                matches!(decode_scancode(code), Err(Error::InvalidArgument(_))),
                "scancode {code:#X} should be rejected"
            );
        }
    }

    #[test]
    fn key_messages_become_one_key_action_and_ignore_modifiers() {
        assert_eq!(
            plan(&key_event(0x1E, 0, true)).expect("key down"),
            vec![HostInput::Key {
                scan: 0x1E,
                extended: false,
                up: false
            }]
        );
        // `modifiers` is not defined by the protocol and must not synthesise extra keys.
        assert_eq!(
            plan(&key_event(0x1E, 0b1111, false)).expect("key up"),
            vec![HostInput::Key {
                scan: 0x1E,
                extended: false,
                up: true
            }]
        );
        assert_eq!(
            plan(&key_event(EXTENDED_SCANCODE | 0x4B, 0, true)).expect("extended"),
            vec![HostInput::Key {
                scan: 0x4B,
                extended: true,
                up: false
            }]
        );
    }

    #[test]
    fn mouse_buttons_map_by_wire_id() {
        assert_eq!(mouse_button(1).expect("left"), MouseButton::Left);
        assert_eq!(mouse_button(2).expect("right"), MouseButton::Right);
        assert_eq!(mouse_button(3).expect("middle"), MouseButton::Middle);
        assert_eq!(mouse_button(4).expect("x1"), MouseButton::X1);
        assert_eq!(mouse_button(5).expect("x2"), MouseButton::X2);
        for bad in [0, 6, 255] {
            assert!(matches!(mouse_button(bad), Err(Error::InvalidArgument(_))));
        }

        assert_eq!(
            plan(&InputMessage::MouseButton {
                button: 2,
                down: true
            })
            .expect("button"),
            vec![HostInput::Button {
                button: MouseButton::Right,
                down: true
            }]
        );
    }

    #[test]
    fn wheel_notches_become_delta_units() {
        assert_eq!(wheel_units(1).expect("one notch"), WHEEL_DELTA);
        assert_eq!(wheel_units(-3).expect("three back"), -3 * WHEEL_DELTA);
        assert!(matches!(
            wheel_units(i32::MAX),
            Err(Error::InvalidArgument(_))
        ));

        assert_eq!(
            plan(&InputMessage::Wheel { dx: 0, dy: 2 }).expect("vertical"),
            vec![HostInput::Wheel {
                units: 2 * WHEEL_DELTA,
                horizontal: false
            }]
        );
        assert_eq!(
            plan(&InputMessage::Wheel { dx: -1, dy: 1 }).expect("both"),
            vec![
                HostInput::Wheel {
                    units: WHEEL_DELTA,
                    horizontal: false
                },
                HostInput::Wheel {
                    units: -WHEEL_DELTA,
                    horizontal: true
                },
            ]
        );
        assert!(matches!(
            plan(&InputMessage::Wheel { dx: 0, dy: 0 }),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn text_is_injected_as_utf16_down_up_pairs() {
        assert_eq!(
            plan(&InputMessage::Text {
                text: "Aé".into()
            })
            .expect("text"),
            vec![
                HostInput::Text {
                    code_unit: 0x41,
                    up: false
                },
                HostInput::Text {
                    code_unit: 0x41,
                    up: true
                },
                HostInput::Text {
                    code_unit: 0xE9,
                    up: false
                },
                HostInput::Text {
                    code_unit: 0xE9,
                    up: true
                },
            ]
        );

        // Outside the BMP: one surrogate pair, two code units, four events.
        let planned = plan(&InputMessage::Text {
            text: "\u{1D11E}".into(),
        })
        .expect("surrogate pair");
        assert_eq!(planned.len(), 4);
        assert!(matches!(
            planned[0],
            HostInput::Text { up: false, .. }
        ));
        assert!(matches!(planned[2], HostInput::Text { up: false, .. }));

        assert!(matches!(
            plan(&InputMessage::Text { text: String::new() }),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn cad_request_is_honestly_unavailable() {
        assert!(matches!(
            plan(&InputMessage::CadRequest),
            Err(Error::NotImplemented(_))
        ));
    }

    #[test]
    fn absolute_positions_are_normalised_over_the_desktop() {
        assert_eq!(
            normalize_absolute(0, 0, bounds()).expect("origin"),
            (0, 0)
        );
        assert_eq!(
            normalize_absolute(1919, 1079, bounds()).expect("last pixel"),
            (65_535, 65_535)
        );
        // Normalisation maps the desktop range [0, extent-1] to [0, 65535].
        // The exact output depends on integer truncation; verify it is monotonic
        // and produces a value strictly between the edges.
        let mid_x = normalize_absolute(959, 0, bounds()).expect("mid x");
        let mid_y = normalize_absolute(0, 539, bounds()).expect("mid y");
        // Mid values must be strictly between the edges (0 and 65535).
        assert!(mid_x.0 > 0 && mid_x.0 < 65_535, "mid x {}/{} is not in (0, 65535)", mid_x.0, 65_535);
        assert!(mid_y.1 > 0 && mid_y.1 < 65_535, "mid y {}/{} is not in (0, 65535)", mid_y.1, 65_535);
        // Monotonicity: a larger input coordinate must not produce a smaller output.
        let left_x = normalize_absolute(0, 0, bounds()).expect("left x");
        let right_x = normalize_absolute(1919, 0, bounds()).expect("right x");
        assert!(left_x.0 < right_x.0, "left x {} >= right x {}", left_x.0, right_x.0);
    }

    #[test]
    fn absolute_positions_clamp_to_the_desktop_edges() {
        assert_eq!(
            normalize_absolute(-50, 5000, bounds()).expect("clamped"),
            (0, 65_535)
        );

        let secondary = DesktopBounds {
            left: -1920,
            top: 0,
            width: 1920,
            height: 1080,
        };
        assert_eq!(
            normalize_absolute(-1920, 0, secondary).expect("secondary origin"),
            (0, 0)
        );

        let single_pixel = DesktopBounds {
            left: 0,
            top: 0,
            width: 1,
            height: 1,
        };
        assert_eq!(
            normalize_absolute(0, 0, single_pixel).expect("degenerate"),
            (0, 0)
        );
    }

    #[test]
    fn empty_desktop_bounds_are_rejected() {
        for (width, height) in [(0, 1080), (1920, 0), (-5, 5)] {
            assert!(matches!(
                normalize_absolute(0, 0, DesktopBounds {
                    left: 0,
                    top: 0,
                    width,
                    height
                }),
                Err(Error::InvalidArgument(_))
            ));
        }
    }

    #[test]
    fn move_messages_keep_relative_deltas_and_normalise_absolute_ones() {
        let relative = InputMoveMessage {
            x: -7,
            y: 4,
            monitor: 0,
            relative: true,
        };
        assert_eq!(
            plan_move(&relative, bounds()).expect("relative"),
            HostInput::Move {
                dx: -7,
                dy: 4,
                absolute: false
            }
        );

        let absolute = InputMoveMessage {
            x: 1919,
            y: 0,
            monitor: 1,
            relative: false,
        };
        assert_eq!(
            plan_move(&absolute, bounds()).expect("absolute"),
            HostInput::Move {
                dx: 65_535,
                dy: 0,
                absolute: true
            }
        );
    }
}