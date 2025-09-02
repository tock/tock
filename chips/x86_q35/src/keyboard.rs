// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! PS/2 keyboard device skeleton over the i8042 controller.

#![allow(dead_code)] // ONLY FOR THIS MILESTONE SKELETON, WILL BE REMOVED

use crate::ps2::Ps2Client;
use crate::ps2::Ps2Controller;
use core::cell::{Cell, RefCell};
use kernel::debug;
use kernel::utilities::cells::OptionalCell;

/// Set-2 scancode constands used for decoding

const SC_LSHIFT: u8 = 0x12;
const SC_RSHIFT: u8 = 0x59;
const SC_CAPS: u8 = 0x58;

// Minimal, layout-free key event (future commits will populate this)
#[derive(Copy, Clone, Debug)]
pub struct KeyEvent {
    /// Derive key identifier (based on Set-2)
    pub keycode: u16,
    /// true = make (press), false = break (release)
    pub pressed: bool,
    /// true if the event came via the E0 prefix (extended)
    pub extended: bool,
}

/// Optional byte send for ASCII output (useful for a later capsule)
pub trait AsciiClient {
    fn put_byte(&self, b: u8);
}

/// Callback that the keyboard will use to deliver events
pub trait KeyboardClient {
    fn key_event(&self, _ev: KeyEvent) {
        // TODO
    }
}

/// We will add a small "command engine" (command/response state machine)
/// with ACK/RESEND handling.
/// A fixed-size FIFO holds short command sequences (e.g., F0 02, F4).
/// One byte is in flight at a time; 0xFA ACK advances, 0xFE RESEND retries.

const CMDQ_LEN: usize = 8;
const CMD_MAX_LEN: usize = 3;
const MAX_RETRIES: u8 = 3; // to not be confused with the deff call "good bytes" we do for telemetry

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
}

/// We capture bytes and will later add:
/// 1) Command FIFO with ACK/RESEND handling
/// 2) Set-2 decoder state machine (e0/f0/e1)
/// Modifier tracking and ASCII/layout mapping in upper layers

pub struct Keyboard<'a> {
    ps2: &'a Ps2Controller,
    client: OptionalCell<&'static dyn KeyboardClient>,
    ascii: OptionalCell<&'static dyn AsciiClient>,

    // decoder state
    got_e0: Cell<bool>,   // sau 0xE0: next code is extended
    got_f0: Cell<bool>,   // saw oxF0: next code is a break
    swallow_e1: Cell<u8>, // > 0 means we are swallowing remaining Pause seq bytes

    // modifiers
    shift_l: Cell<bool>,
    shift_r: Cell<bool>,
    caps: Cell<bool>,

    // diagnostics
    bytes_seen: Cell<u32>,

    // here comes the engine
    cmd_q: RefCell<[CmdEntry; CMDQ_LEN]>,
    q_head: Cell<usize>,  // write cursor
    q_tail: Cell<usize>,  // read cursor
    q_count: Cell<usize>, // number of entries enqueued

    in_flight: Cell<bool>,      // waiting for ACK/RESEND to the last sent byte
    retries_left: Cell<u8>,     // remaining entries for the current byte
    cmd_sent_bytes: Cell<u32>,  //bytes attempted to send
    cmd_acks: Cell<u32>,        // ACKs observed
    cmd_resends: Cell<u32>,     // RESENDs observed
    cmd_drops: Cell<u32>,       //commands dropped after retry execution
    cmd_send_errors: Cell<u32>, // controller TX errors/timeouts
}

impl<'a> Keyboard<'a> {
    pub const fn new(ps2: &'a Ps2Controller) -> Self {
        Self {
            ps2,
            client: OptionalCell::empty(),
            ascii: OptionalCell::empty(),

            // decoder state
            got_e0: Cell::new(false),
            got_f0: Cell::new(false),
            swallow_e1: Cell::new(0),
            shift_l: Cell::new(false),
            shift_r: Cell::new(false),
            caps: Cell::new(false),
            bytes_seen: Cell::new(0),

            cmd_q: RefCell::new([CmdEntry::empty(); CMDQ_LEN]),
            q_head: Cell::new(0),
            q_tail: Cell::new(0),
            q_count: Cell::new(0),

            in_flight: Cell::new(false),
            retries_left: Cell::new(0),

            cmd_sent_bytes: Cell::new(0),
            cmd_acks: Cell::new(0),
            cmd_resends: Cell::new(0),
            cmd_drops: Cell::new(0),
            cmd_send_errors: Cell::new(0),
        }
    }

    /// Install the client which will receive the events
    pub fn set_client(&self, client: &'static dyn KeyboardClient) {
        self.client.set(client);
    }

    /// Device-level init hook. No-op for now since the `init_early()`
    /// already is done by the controller
    pub fn init_device(&self) {
        // TODO
    }

    pub fn set_ascii_client(&self, c: &'static dyn AsciiClient) {
        self.ascii.set(c);
    }

    /// Translate a Set-2 scancode to US-ASCII (press only, non-extend)
    /// Returns None for unmapped keys
    #[inline(always)]
    fn ascii_for(&self, code: u8, pressed: bool, extended: bool) -> Option<u8> {
        if !pressed || extended {
            return None;
        }
        // modifier states
        let shift = self.shift_l.get() || self.shift_r.get();
        let caps = self.caps.get();

        // letters
        let letter = match code {
            0x1C => b'a',
            0x32 => b'b',
            0x21 => b'c',
            0x23 => b'd',
            0x24 => b'e',
            0x2B => b'f',
            0x34 => b'g',
            0x33 => b'h',
            0x43 => b'i',
            0x3B => b'j',
            0x42 => b'k',
            0x4B => b'l',
            0x3A => b'm',
            0x31 => b'n',
            0x44 => b'o',
            0x4D => b'p',
            0x15 => b'q',
            0x2D => b'r',
            0x1B => b's',
            0x2C => b't',
            0x3C => b'u',
            0x2A => b'v',
            0x1D => b'w',
            0x22 => b'x',
            0x35 => b'y',
            0x1A => b'z',
            _ => 0,
        };
        if letter != 0 {
            let upper = shift ^ caps;
            return Some(if upper {
                letter.to_ascii_uppercase()
            } else {
                letter
            });
        }

        // Digits
        let digit = match code {
            0x16 => (b'1', b'!'),
            0x1E => (b'2', b'@'),
            0x26 => (b'3', b'#'),
            0x25 => (b'4', b'$'),
            0x2E => (b'5', b'%'),
            0x36 => (b'6', b'^'),
            0x3D => (b'7', b'&'),
            0x3E => (b'8', b'*'),
            0x46 => (b'9', b'('),
            0x45 => (b'0', b')'),
            _ => (0, 0),
        };
        if digit.0 != 0 {
            return Some(if shift { digit.1 } else { digit.0 });
        }

        // Punctuation / misc (US)
        let punct = match code {
            0x0E => (b'`', b'~'),
            0x4E => (b'-', b'_'),
            0x55 => (b'=', b'+'),
            0x54 => (b'[', b'{'),
            0x5B => (b']', b'}'),
            0x5D => (b'\\', b'|'),
            0x4C => (b';', b':'),
            0x52 => (b'\'', b'"'),
            0x41 => (b',', b'<'),
            0x49 => (b'.', b'>'),
            0x4A => (b'/', b'?'),
            0x29 => (b' ', b' '),   // space
            0x0D => (b'\t', b'\t'), // tab
            0x5A => (b'\n', b'\n'), // enter
            0x66 => (0x08, 0x08),   // backspace
            _ => (0, 0),
        };
        if punct.0 != 0 {
            return Some(if shift { punct.1 } else { punct.0 });
        }

        None
    }

    /// Command engine public API
    ///
    /// Enqueue a short command sequence
    /// Returns false if the queue is full or seq is too long

    pub fn enqueue_command(&self, seq: &[u8]) -> bool {
        if seq.is_empty() || seq.len() > CMD_MAX_LEN {
            return false;
        }
        if self.q_count.get() >= CMDQ_LEN {
            return false;
        }
        // Copy into the queue at head
        let head = self.q_head.get();
        {
            let mut q = self.cmd_q.borrow_mut();
            let e = &mut q[head];
            e.bytes = [0; CMD_MAX_LEN];
            e.bytes[..seq.len()].copy_from_slice(seq);
            e.len = seq.len() as u8;
            e.idx = 0;
        }
        self.q_head.set((head + 1) % CMDQ_LEN);
        self.q_count.set(self.q_count.get() + 1);
        self.drive_tx();

        true
    }

    /// Try to transmit the next byte of the current command
    fn drive_tx(&self) {
        if self.in_flight.get() || self.q_count.get() == 0 {
            return;
        }

        // Peek current entry at tail and the next byte to send
        let (byte_opt, done) = {
            let q = self.cmd_q.borrow();
            let e = &q[self.q_tail.get()];
            if e.is_done() {
                (None, true)
            } else {
                (Some(e.bytes[e.idx as usize]), false)
            }
        };

        if done {
            // This shouldn't persist-pop and try again
            self.pop_cmd();
            self.drive_tx();
            return;
        }

        if let Some(b) = byte_opt {
            // Attempt to send. If the controller times out, do not mark inflight,
            // so a later call may retry. We also don't advance idx here, only on ACK
            match self.ps2.send_port1(b) {
                Ok(()) => {
                    self.in_flight.set(true);
                    if self.retries_left.get() == 0 {
                        // first attempt for this byte
                        self.retries_left.set(MAX_RETRIES);
                    }
                    self.cmd_sent_bytes
                        .set(self.cmd_sent_bytes.get().wrapping_add(1));
                }

                Err(_e) => {
                    // Controller busy/timeout; count and let a later tick retry
                    self.cmd_send_errors
                        .set(self.cmd_send_errors.get().wrapping_add(1));
                }
            }
        }
    }

    fn pop_cmd(&self) {
        if self.q_count.get() == 0 {
            return;
        }
        self.q_tail.set((self.q_tail.get() + 1) % CMDQ_LEN);
        self.q_count.set(self.q_count.get() - 1);
    }

    fn advance_idx_after_ack(&self) {
        // Increment idx of the current entry; if complete, pop
        let mut finished = false;
        {
            let mut q = self.cmd_q.borrow_mut();
            let e = &mut q[self.q_tail.get()];
            if !e.is_done() {
                e.idx = e.idx.saturating_add(1)
            }
            if e.is_done() {
                finished = true;
            }
        }
        if finished {
            self.pop_cmd();
        }

        // New byte will get a fresh budget retry on first send
        self.retries_left.set(0);
    }

    #[inline(always)]
    fn emit_event(&self, code: u8, pressed: bool, extended: bool) {
        let keycode = (code as u16) | (if extended { 0x0100 } else { 0x0000 });
        let ev = KeyEvent {
            keycode,
            pressed,
            extended,
        };

        // deliver to client if present, else log something
        if self.client.is_some() {
            self.client.map(|c| c.key_event(ev));
        } else {
            if extended {
                debug!(
                    "ps2-kbd: EV {} E0 {:02X}",
                    if pressed { "MAKE " } else { "BREAK" },
                    code
                );
            } else {
                debug!(
                    "ps2-kbd: EV {} {:02X}",
                    if pressed { "MAKE " } else { "BREAK" },
                    code
                );
            }
        }
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

        // Update local modifier state (we still emit events for them)
        // consumers can ignore
        match byte {
            SC_LSHIFT => self.shift_l.set(pressed),
            SC_RSHIFT => self.shift_r.set(pressed),
            SC_CAPS if pressed => self.caps.set(!self.caps.get()), // toggle on make; ignore break
            _ => {}
        }

        // Emit event for this key
        self.emit_event(byte, pressed, extended);

        if let Some(b) = self.ascii_for(byte, pressed, extended) {
            if self.ascii.is_some() {
                self.ascii.map(|c| c.put_byte(b));
            } else {
                // Logging
                if b >= 0x20 && b != 0x7F {
                    debug!("pse-kbd: ASCII '{}'", b as char);
                } else {
                    debug!("pse-kbd: ASCII 0x{:02x}", b);
                }
            }
        }
    }
}
impl Ps2Client for Keyboard<'_> {
    /// Called by the controller (in def context) for each byte)
    fn receive_scancode(&self, byte: u8) {
        // First, if a command byte is in flight, interpret 0XFA/0XFE
        if self.in_flight.get() {
            match byte {
                0xFA => {
                    // ACK
                    self.cmd_acks.set(self.cmd_acks.get().wrapping_add(1));
                    self.in_flight.set(false);
                    self.advance_idx_after_ack();
                    // Immediately try to send the next byte/command
                    self.drive_tx();
                    return;
                }
                0xFE => {
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
                        self.pop_cmd();
                        self.retries_left.set(0); // clear for the next new byte
                        self.drive_tx();
                    }
                    return;
                }
                _ => {
                    // Not an ACK/RESEND. For now we ignore it
                    // Keep waiting for ACK/RESEND
                }
            }
        }
        self.decode_byte(byte);

        // Optional: basic init + counter
        let n = self.bytes_seen.get().wrapping_add(1);
        self.bytes_seen.set(n);

        // keep the log basic
        if n <= 8 || (n & 0x0F) == 0 {
            debug!("ps2-kbd: byte {:02x} (count={})", byte, n);
        }

        // Decoder and event emitter come in the future
        let _ = &self.ps2;
        let _ = &self.client;
    }
}
