// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! PS/2 keyboard device over the i8042 controller.
//!
//! How it works (overview)
//! - I8042 (ports available on 0X60/0X64) delivers keyboard bytes via IRQ1
//! - We use scan Code set 2 (no translation). 0xE0/0xE1 are prefixes; 0xF0 marks BREAK
//! - Keyboard speaks simple command/ACK (0xFA) / RESEND (0xFE) protocol
//! References:
//! - OSDev: i8042 PS/2 Controller — https://wiki.osdev.org/I8042_PS/2_Controller
//! - OSDev: PS/2 Keyboard — https://wiki.osdev.org/PS/2_Keyboard
//! - OSDev: Keyboard / Scan Code Set 2 — https://wiki.osdev.org/Keyboard#Scan_Code_Set_2

use crate::cmd_fifo::Fifo as CmdFifo;
use crate::ps2::Ps2Client;
use crate::ps2::Ps2Controller;
use core::cell::Cell;
use kernel::debug;
use kernel::hil::keyboard::{Keyboard as HilKeyboard, KeyboardClient as HilKeyboardClient};
use kernel::utilities::cells::OptionalCell;

/// Set-2 - Linux keycode mapper
#[inline(always)]
fn linux_key_for(extended: bool, code: u8) -> Option<u16> {
    match (extended, code) {
        // basics
        (false, 0x76) => Some(1),  // KEY_ESC
        (false, 0x5A) => Some(28), // KEY_ENTER
        (false, 0x66) => Some(14), // KEY_BACKSPACE
        (false, 0x0D) => Some(15), // KEY_TAB
        (false, 0x29) => Some(57), // KEY_SPACE

        // modifiers
        (false, 0x12) => Some(42), // KEY_LEFTSHIFT
        (false, 0x59) => Some(54), // KEY_RIGHTSHIFT
        (false, 0x14) => Some(29), // KEY_LEFTCTRL
        (true, 0x14) => Some(97),  // KEY_RIGHTCTRL
        (false, 0x11) => Some(56), // KEY_LEFTALT
        (true, 0x11) => Some(100), // KEY_RIGHTALT (AltGr)
        (false, 0x58) => Some(58), // KEY_CAPSLOCK

        // arrows (E0)
        (true, 0x75) => Some(103), // KEY_UP
        (true, 0x72) => Some(108), // KEY_DOWN
        (true, 0x6B) => Some(105), // KEY_LEFT
        (true, 0x74) => Some(106), // KEY_RIGHT

        // letters (US)
        (false, 0x1C) => Some(30), // A
        (false, 0x32) => Some(48), // B
        (false, 0x21) => Some(46), // C
        (false, 0x23) => Some(32), // D
        (false, 0x24) => Some(18), // E
        (false, 0x2B) => Some(33), // F
        (false, 0x34) => Some(34), // G
        (false, 0x33) => Some(35), // H
        (false, 0x43) => Some(23), // I
        (false, 0x3B) => Some(36), // J
        (false, 0x42) => Some(37), // K
        (false, 0x4B) => Some(38), // L
        (false, 0x3A) => Some(50), // M
        (false, 0x31) => Some(49), // N
        (false, 0x44) => Some(24), // O
        (false, 0x4D) => Some(25), // P
        (false, 0x15) => Some(16), // Q
        (false, 0x2D) => Some(19), // R
        (false, 0x1B) => Some(31), // S
        (false, 0x2C) => Some(20), // T
        (false, 0x3C) => Some(22), // U
        (false, 0x2A) => Some(47), // V
        (false, 0x1D) => Some(17), // W
        (false, 0x22) => Some(45), // X
        (false, 0x35) => Some(21), // Y
        (false, 0x1A) => Some(44), // Z

        // digits row (US)
        (false, 0x16) => Some(2),  // 1
        (false, 0x1E) => Some(3),  // 2
        (false, 0x26) => Some(4),  // 3
        (false, 0x25) => Some(5),  // 4
        (false, 0x2E) => Some(6),  // 5
        (false, 0x36) => Some(7),  // 6
        (false, 0x3D) => Some(8),  // 7
        (false, 0x3E) => Some(9),  // 8
        (false, 0x46) => Some(10), // 9
        (false, 0x45) => Some(11), // 0

        _ => None,
    }
}

/// Set-2 scancode constants used for decoding

const SC_LSHIFT: u8 = 0x12;
const SC_RSHIFT: u8 = 0x59;
const SC_CAPS: u8 = 0x58;
const RESP_ACK: u8 = 0xFA;
const RESP_RESEND: u8 = 0xFE;
const RESP_BAT_OK: u8 = 0xAA; // BAT after reset

/// We will add a small "command engine" (command/response state machine)
/// with ACK/RESEND handling.
/// A fixed-size FIFO holds short command sequences (e.g., F0 02, F4).
/// One byte is in flight at a time; 0xFA ACK advances, 0xFE RESEND retries.

const CMDQ_LEN: usize = 8;
const CMD_MAX_LEN: usize = 3;
const MAX_RETRIES: u8 = 3; // to not be confused with the deff call "good bytes" we do for telemetry on controller

#[derive(Copy, Clone)]
struct CmdEntry {
    bytes: [u8; CMD_MAX_LEN],
    len: u8,
    idx: u8, //next byte to send
}

impl CmdEntry {
    const fn empty() -> Self {
        Self {
            bytes: [0; CMD_MAX_LEN],
            len: 0,
            idx: 0,
        }
    }

    fn is_done(&self) -> bool {
        (self.idx as usize) >= (self.len as usize)
    }

    // helper to avoid duplicating the “turn a byte slice into a queued command” logic
    fn try_from_bytes(seq: &[u8]) -> Option<Self> {
        if seq.is_empty() || seq.len() > CMD_MAX_LEN {
            return None;
        }
        let mut e = CmdEntry::empty();
        e.bytes[..seq.len()].copy_from_slice(seq);
        e.len = seq.len() as u8;
        Some(e)
    }
}

// this is needed so CmdEntry expects a FifoItem, and not a const default
impl crate::cmd_fifo::FifoItem for CmdEntry {
    const EMPTY: Self = Self::empty();
}

impl Default for CmdEntry {
    fn default() -> Self {
        CmdEntry::empty()
    }
}

/// We capture bytes and will later add:
/// 1) Command FIFO with ACK/RESEND handling
/// 2) Set-2 decoder state machine (e0/f0/e1)
/// Modifier tracking and ASCII/layout mapping in upper layers

pub struct Keyboard<'a> {
    ps2: &'a Ps2Controller,
    client: OptionalCell<&'a dyn HilKeyboardClient>,

    // decoder state
    got_e0: Cell<bool>,   // sau 0xE0: next code is extended
    got_f0: Cell<bool>,   // saw 0xF0: next code is a break
    swallow_e1: Cell<u8>, // > 0 means we are swallowing remaining Pause seq bytes

    // modifiers
    shift_l: Cell<bool>,
    shift_r: Cell<bool>,
    caps: Cell<bool>,

    // command engine queue (ring FIFO) - interior mutable via Cells inside
    cmd_q: CmdFifo<CmdEntry, CMDQ_LEN>,

    in_flight: Cell<bool>,      // waiting for ACK/RESEND to the last sent byte
    retries_left: Cell<u8>,     // remaining entries for the current byte
    cmd_sent_bytes: Cell<u32>,  //bytes attempted to send
    cmd_acks: Cell<u32>,        // ACKs observed
    cmd_resends: Cell<u32>,     // RESENDs observed
    cmd_drops: Cell<u32>,       //commands dropped after retry execution
    cmd_send_errors: Cell<u32>, // controller TX errors/timeouts
}

impl<'a> HilKeyboard<'a> for Keyboard<'a> {
    fn set_client(&self, client: &'a dyn HilKeyboardClient) {
        self.client.set(client);
    }
}

impl<'a> Keyboard<'a> {
    pub const fn new(ps2: &'a Ps2Controller) -> Self {
        Self {
            ps2,
            client: OptionalCell::empty(),

            // decoder state
            got_e0: Cell::new(false),
            got_f0: Cell::new(false),
            swallow_e1: Cell::new(0),
            shift_l: Cell::new(false),
            shift_r: Cell::new(false),
            caps: Cell::new(false),

            cmd_q: CmdFifo::new(),

            in_flight: Cell::new(false),
            retries_left: Cell::new(0),

            cmd_sent_bytes: Cell::new(0),
            cmd_acks: Cell::new(0),
            cmd_resends: Cell::new(0),
            cmd_drops: Cell::new(0),
            cmd_send_errors: Cell::new(0),
        }
    }

    /// Device-level init hook. No-op for now since the `init_early()`
    /// already runs in the controller. After capsule lands, we will enqueue:
    ///  F5 (disable scan) -> FF (reset; expect FA then AA) - F0 02 (Set-2) - F4 (enable).
    /// This will use the command engine (ACK/RESEND) and can run with IRQ1 enabled.
    pub fn init_device(&self) {
        // TODO
    }

    /// Command engine public API
    ///
    /// Queue a short PS/2 device command (e.g. `&[0xF0, 0x02]`, `&[0xF4]`).
    ///
    ///  Runs in the keyboard’s deferred/bottom-half context;
    ///  Bytes are sent one-by-one; `ACK (0xFA)` advances; `RESEND (0xFE)` retries with a bounded budget.
    ///  Returns `false` if the queue is full or the sequence length exceeds `CMD_MAX_LEN`.
    ///  No completion callback; failures are tracked internally (telemetry counters).

    pub fn enqueue_command(&self, seq: &[u8]) -> bool {
        let Some(entry) = CmdEntry::try_from_bytes(seq) else {
            return false;
        };
        if self.cmd_q.is_full() {
            return false;
        }
        let _ = self.cmd_q.push(entry);
        self.drive_tx();
        true
    }

    /// Try to transmit the next byte of the current command
    fn drive_tx(&self) {
        if self.in_flight.get() {
            return;
        }

        // Peek current command
        let (byte_opt, done_opt) = match self.cmd_q.peek_copy() {
            None => (None, None),
            Some(e) if e.is_done() => (None, Some(true)),
            Some(e) => (Some(e.bytes[e.idx as usize]), Some(false)),
        };

        match done_opt {
            None => return, // queue empty
            Some(true) => {
                // finished entry => pop and try next
                self.cmd_q.pop();
                self.drive_tx();
                return;
            }
            Some(false) => {}
        }

        if let Some(b) = byte_opt {
            match self.ps2.send_port1(b) {
                Ok(()) => {
                    self.in_flight.set(true);
                    if self.retries_left.get() == 0 {
                        self.retries_left.set(MAX_RETRIES);
                    }
                    self.cmd_sent_bytes
                        .set(self.cmd_sent_bytes.get().wrapping_add(1));
                }
                Err(_e) => {
                    self.cmd_send_errors
                        .set(self.cmd_send_errors.get().wrapping_add(1));
                }
            }
        }
    }

    fn advance_idx_after_ack(&self) {
        let finished = self
            .cmd_q
            .peek_update(|e| {
                if !e.is_done() {
                    e.idx = e.idx.saturating_add(1);
                }
                e.is_done()
            })
            .unwrap_or(false);
        if finished {
            self.cmd_q.pop();
        }
        // New byte will get a fresh retry budget on first send
        self.retries_left.set(0);
    }

    /// Core set-2 decoder. Consumes one non-command byte
    #[inline(always)]
    fn decode_byte(&self, byte: u8) {
        // handle E1 (pause) long sequence
        if self.swallow_e1.get() > 0 {
            self.swallow_e1.set(self.swallow_e1.get() - 1);
            return;
        }
        if byte == 0xE1 {
            // start swallowing the remaining 7 bytes
            self.swallow_e1.set(7);
            return;
        }

        // Prefix events (caps, shifts)
        if byte == 0xE0 {
            self.got_e0.set(true);
            return;
        }

        if byte == 0xF0 {
            self.got_f0.set(true);
            return;
        }

        // resolve latched prefixes
        let extended = self.got_e0.replace(false);
        let breaking = self.got_f0.replace(false);
        let pressed = !breaking;

        // log the final key event (after prefixes are applied)
        debug!(
            "ps2kbd: {} {}{:02X}",
            if pressed { "MAKE " } else { "BREAK" },
            if extended { "E0 " } else { "" },
            byte
        );

        // Emit Linux keycode to the HIL client
        if let Some(code) = linux_key_for(extended, byte) {
            self.client.map(|c| {
                let one = [(code, pressed)];
                c.keys_pressed(&one, Ok(()));
            });
        }

        // Update local modifier state (we still emit events for them)
        // consumers can ignore
        match (byte, extended) {
            (SC_LSHIFT, false) => self.shift_l.set(pressed),
            (SC_RSHIFT, false) => self.shift_r.set(pressed),
            (SC_CAPS, _) if pressed => self.caps.set(!self.caps.get()), // toggle on make; ignore break
            _ => {}
        }
    }
}
impl Ps2Client for Keyboard<'_> {
    /// Called by the controller (in def context) for each byte)
    fn receive_scancode(&self, byte: u8) {
        // First, if a command byte is in flight, interpret 0XFA/0XFE
        if self.in_flight.get() {
            match byte {
                RESP_ACK => {
                    // ACK
                    self.cmd_acks.set(self.cmd_acks.get().wrapping_add(1));
                    self.in_flight.set(false);
                    self.advance_idx_after_ack();
                    // Immediately try to send the next byte/command
                    self.drive_tx();
                    return;
                }
                RESP_RESEND => {
                    // RESEND - bounded retry of the same byte
                    self.cmd_resends.set(self.cmd_resends.get().wrapping_add(1));
                    let left = self.retries_left.get();
                    if left > 1 {
                        self.retries_left.set(left - 1);
                        self.in_flight.set(false);
                        self.drive_tx(); // resend same byte, don't reset retries
                    } else {
                        // Give up on this command
                        self.cmd_drops.set(self.cmd_drops.get().wrapping_add(1));
                        self.in_flight.set(false);
                        self.cmd_q.pop();
                        self.retries_left.set(0); // clear for the next new byte
                        self.drive_tx();
                    }
                    return;
                }
                _ => {
                    // Not an ACK/RESEND while waiting = ignore (do NOT decode as a key).
                    return;
                }
            }
        }
        // No command in flight: ignore device-only responses that aren’t keystrokes.
        if byte == RESP_ACK || byte == RESP_RESEND || byte == RESP_BAT_OK {
            return;
        }
        self.decode_byte(byte);
    }
}
