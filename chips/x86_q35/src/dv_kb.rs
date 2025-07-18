// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! PS/2 keyboard wrapper and Set‑2 decoder for the 8042 controller
use core::cell::RefCell;
use core::marker::PhantomData;
use kernel::hil::ps2_traits::{KBReceiver, PS2Keyboard, PS2Traits};
use kernel::errorcode::ErrorCode;

/// Public key‑event types

/// High‑level keyboard event exposed to capsules.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyEvent {
    /// Printable ASCII (already affected by Shift / CapsLock).
    Ascii(u8),
    /// A few non‑printing keys that text UIs care about.
    Special(SpecialKey),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpecialKey {
    Backspace,
    Enter,
    Tab,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    PauseBreak,
}

// Single‑producer / single‑consumer ring buffer for events
const EVT_CAP: usize = 64; // power‑of‑two not required here

struct EventFifo {
    buf: [Option<KeyEvent>; EVT_CAP],
    head: usize, // write position
    tail: usize, // read position
    full: bool,
}

impl EventFifo {
    const fn new() -> Self {
        Self {
            buf: [None; EVT_CAP],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    /// Push, overwriting the oldest event on overflow (lossy but simple).
    #[inline]
    fn push(&mut self, ev: KeyEvent) {
        self.buf[self.head] = Some(ev);
        self.head = (self.head + 1) % EVT_CAP;
        if self.full {
            self.tail = (self.tail + 1) % EVT_CAP; // drop oldest
        } else if self.head == self.tail {
            self.full = true;
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<KeyEvent> {
        if !self.full && self.head == self.tail {
            return None; // empty
        }
        let ev = self.buf[self.tail].take();
        self.tail = (self.tail + 1) % EVT_CAP;
        self.full = false;
        ev
    }
}

/// Internal decoder state.  Only make codes generate output.
pub struct DecoderState {
    prefix_e0: bool,
    prefix_e1: u8, // 0 = none, 1 = got E1, 2 = got E1 14
    break_code: bool,

    // Modifier latches
    shift: bool,
    caps_lock: bool,
}

impl DecoderState {
    /// Fresh decoder (modifiers cleared).
    pub const fn new() -> Self {
        Self {
            prefix_e0: false,
            prefix_e1: 0,
            break_code: false,
            shift: false,
            caps_lock: false,
        }
    }

    /// Feed one raw scan‑code byte; returns Some(KeyEvent) on make only.
    #[inline]
    pub fn process(&mut self, raw: u8) -> Option<KeyEvent> {
        // ---------------- Prefix handling ----------------
        if raw == 0xE0 {
            self.prefix_e0 = true;
            return None;
        }
        if raw == 0xE1 {
            self.prefix_e1 = 1;
            return None;
        }
        if self.prefix_e1 != 0 {
            // Only handle Pause/Break (E1 14 77 …)
            match (self.prefix_e1, raw) {
                (1, 0x14) => {
                    self.prefix_e1 = 2;
                    return None;
                }
                (2, 0x77) => {
                    self.prefix_e1 = 0;
                    return Some(KeyEvent::Special(SpecialKey::PauseBreak));
                }
                _ => {
                    self.prefix_e1 = 0; // unrecognised sequence
                    return None;
                }
            }
        }

        if raw == 0xF0 {
            self.break_code = true;
            return None;
        }

        // At this point `raw` is a make/break code depending on flag.
        let make = !self.break_code;
        self.break_code = false;

        // Modifier latches
        match raw {
            0x12 | 0x59 => {
                // Shift (both sides)
                self.shift = make;
                return None;
            }
            0x58 => {
                // CapsLock toggles on make only
                if make {
                    self.caps_lock = !self.caps_lock;
                }
                return None;
            }
            _ => {}
        }

        if !make {
            return None; // ignore releases for non‑modifier keys
        }

        // Make => KeyEvent
        if let Some(ascii) = map_scan_to_ascii(raw, self.shift ^ self.caps_lock) {
            match ascii {
                b'\n' => return Some(KeyEvent::Special(SpecialKey::Enter)),
                0x08 => return Some(KeyEvent::Special(SpecialKey::Backspace)),
                b'\t' => return Some(KeyEvent::Special(SpecialKey::Tab)),
                _ => return Some(KeyEvent::Ascii(ascii)),
            }
        }

        // Arrow keys are prefixed with E0.
        if self.prefix_e0 {
            self.prefix_e0 = false; // consume prefix
            let sk = match raw {
                0x75 => SpecialKey::ArrowUp,
                0x72 => SpecialKey::ArrowDown,
                0x6B => SpecialKey::ArrowLeft,
                0x74 => SpecialKey::ArrowRight,
                _ => return None,
            };
            return Some(KeyEvent::Special(sk));
        }

        None
    }
}

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
    events: RefCell<EventFifo>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, C: PS2Traits> Keyboard<'a, C> {
    pub fn new(ps2: &'a C) -> Self {
        Self {
            ps2,
            decoder: RefCell::new(DecoderState::new()),
            events: RefCell::new(EventFifo::new()),
            _marker: PhantomData,
        }
    }

    /// Thin top‑half: simply forward to the controller.
    pub fn handle_interrupt(&self) {
        let _ = self.ps2.handle_interrupt();
    }
    /// Bottom-half: drain raw bytes and queue KeyEvents
    pub fn poll (&self) {
        while let Some(raw) = self.ps2.pop_scan_code() {
            if let Some(evt) = self.decoder.borrow_mut().process(raw){
                self.events.borrow_mut().push(evt);
            }
        }
    }
    /// Non-blocking getter for consumers
    pub fn next_event(&self) -> Option<KeyEvent> {
        self.events.borrow_mut().pop()
    }
}

impl<'a, C: PS2Traits> KBReceiver for Keyboard<'a, C> {
    /// Return printable ASCII only; drop specials for legacy users.
    fn receive(&self) -> Option<u8> {
        // Drain one raw byte from the controller
        let raw = self.ps2.pop_scan_code()?;
        // Decode it
        match self.decoder.borrow_mut().process(raw) {
            Some(KeyEvent::Ascii(b)) => Some(b),   // forward printable
            _ => None,                             // swallow specials
        }
    }
}


/// Test

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    /// Minimal stub that satisfies `PS2Traits` for unit-testing.
    ///
    /// It implements only the methods the keyboard uses (`pop_scan_code`
    /// and the const fns called by the command helpers are left empty).
    struct DummyPs2 {
        bytes: &'static [u8],
        idx:   Cell<usize>,
    }

    impl DummyPs2 {
        const fn new(bytes: &'static [u8]) -> Self {
            Self { bytes, idx: Cell::new(0) }
        }
    }

    use kernel::errorcode::ErrorCode;

    impl PS2Traits for DummyPs2 {
        /*  methods the tests actually use  */

        fn pop_scan_code(&self) -> Option<u8> {
            let i = self.idx.get();
            if i < self.bytes.len() {
                self.idx.set(i + 1);
                Some(self.bytes[i])
            } else {
                None
            }
        }

        /* signature-correct no-ops  */

        // Controller initialisation
        fn init(&self) {}

        // Host-to-device helpers
        fn wait_input_ready() {}
        fn write_data(_b: u8) {}
        fn write_command(_cmd: u8) {}

        // Device-to-host helper
        fn wait_output_ready() {}
        fn read_data() -> u8 { 0 }

        // IRQ top-half
        fn handle_interrupt(&self) -> Result<(), ErrorCode> {
            Ok(())
        }

        // Push from ISR (unused in unit tests)
        fn push_code(&self, _code: u8) -> Result<(), ErrorCode> {
            Ok(())
        }
    }

    //  Test 1: basic pump path
    #[test]
    fn pump_basic() {
        // ‘a’ press, ‘a’ release (F0 1C)
        static BYTES: &[u8] = &[0x1C, 0xF0, 0x1C];
        let ctl = DummyPs2::new(BYTES);
        let kb  = Keyboard::new(&ctl);

        kb.poll(); // drain all bytes

        assert_eq!(kb.next_event(), Some(KeyEvent::Ascii(b'a')));
        assert_eq!(kb.next_event(), None); // release ignored
    }

    // Test 2: FIFO overflow wraps correctly
    #[test]
    fn overflow() {
        // 70 × ‘a’ presses  -> EVT_CAP = 64, so oldest 6 must drop
        const N: usize = 70;
        static BYTES: [u8; N] = [0x1C; N]; // 70 make codes
        let ctl = DummyPs2::new(&BYTES);
        let kb  = Keyboard::new(&ctl);

        kb.poll();

        // count events left in the FIFO
        let mut n_events = 0;
        while kb.next_event().is_some() {
            n_events += 1;
        }
        assert_eq!(n_events, EVT_CAP); // capped at 64
    }
}
