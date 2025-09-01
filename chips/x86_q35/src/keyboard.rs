// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! PS/2 keyboard device skeleton over the i8042 controller.

#![allow(dead_code)] // ONLY FOR THIS MILESTONE SKELETON, WILL BE REMOVED

use core::cell::Cell;
use kernel::debug;
use kernel::utilities::cells::OptionalCell;
use crate::ps2::Ps2Client;
use crate::ps2::Ps2Controller;

// Minimal, layout-free key event (future commits will populate this)
#[derive(Copy, Clone, Debug)]
pub struct KeyEvent {
    /// Derive key identifier (based on Set-2)
    pub keycode: u16,
    /// true = make (press), false = break (release)
    pub pressed: bool,
    /// true if the event came via the E0 prefix (extended)
    pub extended: bool
}

/// Callback that the keyboard will use to deliver events
pub trait KeyboardClient {
    fn key_event (&self, _ev: KeyEvent) {
        // TODO
    }
}

/// We capture bytes and will later add:
/// 1) Command FIFO with ACK/RESEND handling
/// 2) Set-2 decoder state machine (e0/f0/e1)
/// Modifier tracking and ASCII/layout mapping in upper layers

pub struct Keyboard<'a> {
    ps2: &'a Ps2Controller,
    client: OptionalCell<&'a dyn KeyboardClient>,

    // we only track bytes for now
    bytes_seen: Cell<u32>,
}

impl<'a> Keyboard<'a> {
    pub const fn new(ps2: &'a Ps2Controller) -> Self {
        Self {
            ps2,
            client: OptionalCell::empty(),
            bytes_seen: Cell::new(0),
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
}

impl<'a> Ps2Client for Keyboard<'a> {
    /// Called by the controller (in def context) for each byte)
    fn receive_scancode(&self, byte: u8) {
        // For now: basic init + counter
        let n = self.bytes_seen.get().wrapping_add(1);
        self.bytes_seen.set(n);

        // keep the log basic
        if n<= 8 || (n & 0x0F) == 0 {
            debug!("ps2-kbd: byte {:02x} (count={})", byte, n);
        }

        // Decoder and event emitter come in the future
        // Do not call the keyboardClient yet
        let _ = &self.ps2;
    }
}