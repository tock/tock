// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! PS/2 keyboard wrapper and Set‑2 decoder for the 8042 controller
use core::cell::RefCell;
use core::marker::PhantomData;
use kernel::hil::ps2_traits::{KBReceiver, PS2Keyboard, PS2Traits};

/// Internal decoder state for PS/2 Set2 scan codes.
///
/// The decoder is deliberately *minimal*: it only attempts to turn key **make**
/// events into ASCII bytes for the console capsule.  All key‑release events
/// are consumed to keep modifier state.
pub struct DecoderState {
    prefix_e0: bool,
    prefix_e1: u8, // 0 = none, 1 = got E1, 2 = got E1 14
    break_code: bool,

    // Modifier latches
    shift: bool,
    caps_lock: bool,
}

impl DecoderState {
    /// Create a fresh decoder (all modifiers cleared).
    pub const fn new() -> Self {
        Self {
            prefix_e0: false,
            prefix_e1: 0,
            break_code: false,
            shift: false,
            caps_lock: false,
        }
    }

    /// Feed one raw byte; returns Some(ASCII) only on key‑press.
    #[inline]
    pub fn process(&mut self, raw: u8) -> Option<u8> {
        if raw == 0xE0 {
            self.prefix_e0 = true;
            return None;
        }
        if raw == 0xE1 {
            self.prefix_e1 = 1;
            return None;
        }
        if self.prefix_e1 != 0 {
            // We only care about the Pause/Break make sequence: E1 14 77 …
            match (self.prefix_e1, raw) {
                (1, 0x14) => {
                    self.prefix_e1 = 2;
                    return None;
                }
                (2, 0x77) => {
                    // Emit 0x13 (XOFF) for Pause — no standard ASCII.
                    self.prefix_e1 = 0;
                    return Some(0x13);
                }
                _ => {
                    // Unknown E1 sequence → reset.
                    self.prefix_e1 = 0;
                    return None;
                }
            }
        }

        if raw == 0xF0 {
            // Next byte will be a break code.
            self.break_code = true;
            return None;
        }

        // From here `raw` is a make or break code (depending on flag).
        let make = !self.break_code;
        self.break_code = false;

        // Modifiers first
        // Left & right Shift (E0 12 / 59 for right, 12 for left): we only need
        // the generic “shift” latch for ASCII generation.
        match raw {
            0x12 | 0x59 => {
                if make {
                    self.shift = true;
                } else {
                    self.shift = false;
                }
                return None; // modifiers never emit ASCII
            }
            0x58 => {
                // CapsLock toggles on make only.
                if make {
                    self.caps_lock = !self.caps_lock;
                }
                return None;
            }
            _ => {}
        }

        // Normal keys, boy here we go
        if !make {
            // We ignore all releases for non‑mod keys.
            return None;
        }

        let ascii = map_scan_to_ascii(raw, self.shift ^ self.caps_lock);
        ascii
    }
}

/// Convert a Set‑2 scan code into ASCII.  Returns `None` if the key is not
/// representable (function keys, arrows, etc.).
const fn map_scan_to_ascii(code: u8, shifted: bool) -> Option<u8> {
    match (code, shifted) {
        // Letters
        (0x1C, false) => Some(b'a'), (0x1C, true) => Some(b'A'),
        (0x32, false) => Some(b'b'), (0x32, true) => Some(b'B'),
        (0x21, false) => Some(b'c'), (0x21, true) => Some(b'C'),
        (0x23, false) => Some(b'd'), (0x23, true) => Some(b'D'),
        (0x24, false) => Some(b'e'), (0x24, true) => Some(b'E'),
        (0x2B, false) => Some(b'f'), (0x2B, true) => Some(b'F'),
        (0x34, false) => Some(b'g'), (0x34, true) => Some(b'G'),
        (0x33, false) => Some(b'h'), (0x33, true) => Some(b'H'),
        (0x43, false) => Some(b'i'), (0x43, true) => Some(b'I'),
        (0x3B, false) => Some(b'j'), (0x3B, true) => Some(b'J'),
        (0x42, false) => Some(b'k'), (0x42, true) => Some(b'K'),
        (0x4B, false) => Some(b'l'), (0x4B, true) => Some(b'L'),
        (0x3A, false) => Some(b'm'), (0x3A, true) => Some(b'M'),
        (0x31, false) => Some(b'n'), (0x31, true) => Some(b'N'),
        (0x44, false) => Some(b'o'), (0x44, true) => Some(b'O'),
        (0x4D, false) => Some(b'p'), (0x4D, true) => Some(b'P'),
        (0x15, false) => Some(b'q'), (0x15, true) => Some(b'Q'),
        (0x2D, false) => Some(b'r'), (0x2D, true) => Some(b'R'),
        (0x1B, false) => Some(b's'), (0x1B, true) => Some(b'S'),
        (0x2C, false) => Some(b't'), (0x2C, true) => Some(b'T'),
        (0x3C, false) => Some(b'u'), (0x3C, true) => Some(b'U'),
        (0x2A, false) => Some(b'v'), (0x2A, true) => Some(b'V'),
        (0x1D, false) => Some(b'w'), (0x1D, true) => Some(b'W'),
        (0x22, false) => Some(b'x'), (0x22, true) => Some(b'X'),
        (0x35, false) => Some(b'y'), (0x35, true) => Some(b'Y'),
        (0x1A, false) => Some(b'z'), (0x1A, true) => Some(b'Z'),
        // Digits
        (0x45, false) => Some(b'0'), (0x45, true) => Some(b')'),
        (0x16, false) => Some(b'1'), (0x16, true) => Some(b'!'),
        (0x1E, false) => Some(b'2'), (0x1E, true) => Some(b'@'),
        (0x26, false) => Some(b'3'), (0x26, true) => Some(b'#'),
        (0x25, false) => Some(b'4'), (0x25, true) => Some(b'$'),
        (0x2E, false) => Some(b'5'), (0x2E, true) => Some(b'%'),
        (0x36, false) => Some(b'6'), (0x36, true) => Some(b'^'),
        (0x3D, false) => Some(b'7'), (0x3D, true) => Some(b'&'),
        (0x3E, false) => Some(b'8'), (0x3E, true) => Some(b'*'),
        (0x46, false) => Some(b'9'), (0x46, true) => Some(b'('),
        // Punctuation
        (0x0E, false) => Some(b'`'), (0x0E, true) => Some(b'~'),
        (0x4E, false) => Some(b'-'), (0x4E, true) => Some(b'_'),
        (0x55, false) => Some(b'='), (0x55, true) => Some(b'+'),
        (0x54, false) => Some(b'['), (0x54, true) => Some(b'{'),
        (0x5B, false) => Some(b']'), (0x5B, true) => Some(b'}'),
        (0x5D, false) => Some(b'\\'), (0x5D, true) => Some(b'|'),
        (0x4C, false) => Some(b';'), (0x4C, true) => Some(b':'),
        (0x52, false) => Some(b'\''), (0x52, true) => Some(b'"'),
        (0x41, false) => Some(b','), (0x41, true) => Some(b'<'),
        (0x49, false) => Some(b'.'), (0x49, true) => Some(b'>'),
        (0x4A, false) => Some(b'/'), (0x4A, true) => Some(b'?'),
        // Whitespace & control
        (0x29, _) => Some(b' '),   // space (shift has no effect)
        (0x5A, _) => Some(b'\n'), // Enter
        (0x66, _) => Some(0x08),  // Backspace
        (0x0D, _) => Some(b'\t'), // Tab
        _ => None,
    }
}
/// PS/2 Keyboard wrapper using any `PS2Traits` implementer.
pub struct Keyboard<'a, C: PS2Traits> {
    ps2: &'a C,
    decoder: RefCell<DecoderState>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, C: PS2Traits> Keyboard<'a, C> {
    pub fn new(ps2: &'a C) -> Self {
        Self {
            ps2,
            decoder: RefCell::new(DecoderState::new()),
            _marker: PhantomData,
        }
    }

    /// Thin top‑half: simply forward to the controller.
    pub fn handle_interrupt(&self) {
        let _ = self.ps2.handle_interrupt();
    }
}

impl<'a, C: PS2Traits> KBReceiver for Keyboard<'a, C> {
    /// Poll for a decoded byte (if any).
    fn receive(&self) -> Option<u8> {
        if let Some(raw) = self.ps2.pop_scan_code() {
            self.decoder.borrow_mut().process(raw)
        } else {
            None
        }
    }
}

/// Test

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letter_a() {
        let mut d = DecoderState::new();
        // Press 'a'
        assert_eq!(d.process(0x1C), Some(b'a'));
        // Release 'a' (F0 1C)
        assert_eq!(d.process(0xF0), None);
        assert_eq!(d.process(0x1C), None);
    }

    #[test]
    fn capital_a_with_shift() {
        let mut d = DecoderState::new();
        // Press left shift
        assert_eq!(d.process(0x12), None);
        // Press 'a'
        assert_eq!(d.process(0x1C), Some(b'A'));
        // Release 'a'
        d.process(0xF0); d.process(0x1C);
        // Release shift
        d.process(0xF0); d.process(0x12);
    }
}
