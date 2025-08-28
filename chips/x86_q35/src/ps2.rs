// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use core::cell::{Cell, RefCell};
use kernel::debug;
use kernel::utilities::registers::register_bitfields;
use tock_registers::LocalRegisterCopy;
use x86::registers::io;

/// PS/2 controller ports
const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

/// Depth of the scan-code ring buffer
const BUFFER_SIZE: usize = 32;

/// Timeout limit for spin loops
const TIMEOUT_LIMIT: usize = 1_000_000;

// Status-register bits returned by inb(0x64)
register_bitfields![u8,
    pub STATUS [
        OUTPUT_FULL OFFSET(0) NUMBITS(1), // data ready
        INPUT_FULL  OFFSET(1) NUMBITS(1), // input buffer full
    ]
];

/// This will be explained in the future but we need this
/// to separate possible errors/issues so we don't hang the kernel
/// if the controller breaks, for now, we return something on failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ps2Error {
    InitFailed,
}

/// Note: There is no hardware interrupt when the input buffer empties, so we must poll bit 1.
/// See OSDev documentation:
/// https://wiki.osdev.org/I8042_PS/2_Controller#Status_Register
///
/// Block until the controller’s input buffer is empty (ready for a command).
#[inline(always)]
fn wait_input_ready() {
    // Local copy of the status register
    let mut status_reg = LocalRegisterCopy::<u8, STATUS::Register>::new(0);
    // Loop counter to avoid infinite spin
    let mut loops = 0;

    // Continue spinning while the INPUT_FULL bit is set
    while {
        // Fetch latest status from hardware port
        let raw = read_status();

        status_reg.set(raw);

        // Check if controller is still busy (input buffer full)
        status_reg.is_set(STATUS::INPUT_FULL)
    } {
        // Increment our loop counter and bail out on timeout
        loops += 1;
        if loops >= TIMEOUT_LIMIT {
            // We could log a debug here if desired:
            // debug!("ps2: wait_input_ready timed out after {} loops", loops);
            break;
        }
    }
}

/// Data-ready events trigger IRQ1, handled asynchronously in `handle_interrupt()`.
/// See OSDev documentation:
/// https://wiki.osdev.org/I8042_PS/2_Controller#Status_Register
///
/// Block until there is data ready to read in the output buffer.
#[inline(always)]
fn wait_output_ready() {
    // Local copy of the status register
    let mut status_reg = LocalRegisterCopy::<u8, STATUS::Register>::new(0);
    // Loop counter to prevent infinite spin
    let mut loops = 0;

    // Continue spinning while the OUTPUT_FULL bit is *not* set
    while {
        // Read current status from the controller
        let raw = unsafe { io::inb(PS2_STATUS_PORT) };
        status_reg.set(raw);

        // Keep looping if no data is available yet
        !status_reg.is_set(STATUS::OUTPUT_FULL)
    } {
        // Increment loop counter and abort on timeout
        loops += 1;
        if loops >= TIMEOUT_LIMIT {
            // Optionally log a timeout here:
            // debug!("ps2: wait_output_ready timed out after {} loops", loops);
            break;
        }
    }
}

/// Read one byte from the data port (0x60).
#[inline(always)]
fn read_data() -> u8 {
    wait_output_ready();
    unsafe { io::inb(PS2_DATA_PORT) }
}

/// Send a command byte to the controller (port 0x64).
#[inline(always)]
fn write_command(c: u8) {
    wait_input_ready();
    unsafe { io::outb(PS2_STATUS_PORT, c) }
}
/// Write a data byte to the data port (0x60).
#[inline(always)]
fn write_data(d: u8) {
    wait_input_ready();
    unsafe { io::outb(PS2_DATA_PORT, d) }
}
#[inline(always)]
fn read_status() -> u8 {
    unsafe { io::inb(PS2_STATUS_PORT) }
}

/// Send a byte to the keyboard and wait for ACK (`0xFA`).
/// If the device replies RESEND (`0xFE`) we retry **once**.
///
/// Heads-up: this will be modified in the keyboard driver
/// to better handle command requests,
/// this is just a showcase... for now
fn send_with_ack(byte: u8) -> bool {
    for _ in 0..=1 {
        write_data(byte);
        let resp = read_data();
        match resp {
            0xFA => return true, // ACK
            0xFE => continue,    // RESEND -> try again
            _ => return false,   // error
        }
    }
    false
}

/// PS/2 controller driver (the “8042” peripheral)
pub struct Ps2Controller {
    buffer: RefCell<[u8; BUFFER_SIZE]>,
    head: Cell<usize>,
    tail: Cell<usize>,
    count: Cell<usize>, // new field to track number of valid entries
}

impl Ps2Controller {
    pub const fn new() -> Self {
        Self {
            buffer: RefCell::new([0; BUFFER_SIZE]),
            head: Cell::new(0),
            tail: Cell::new(0),
            count: Cell::new(0),
        }
    }

    /// Pure controller + device bring-up.
    /// No logging, no PIC masking/unmasking, no CPU-IRQ enabling. (hopefully)
    /// Called by PcComponent::finalize() (chip layer).
    /// Whole goal of this change is to stop nagging the memory directly
    pub fn init_early(&self) -> Result<(), Ps2Error> {
        // disable both ports
        write_command(0xAD); // disable keyboard (port 1)
        write_command(0xA7); // disable aux/mouse (port 2)

        // flush any stale bytes from the output buffer
        while read_status() & 0x01 != 0 {
            let _ = unsafe { io::inb(PS2_DATA_PORT) };
        }

        // controller self-test: 0xAA -> expect 0x55
        write_command(0xAA);
        wait_output_ready();
        if read_data() != 0x55 {
            return Err(Ps2Error::InitFailed);
        }

        // read config, ensure IRQ1 off during tests
        write_command(0x20); // read config
        wait_output_ready();
        let mut cfg = read_data();
        cfg &= !(1 << 0); // IRQ1 = 0 (off)
        write_command(0x60); // write config
        write_data(cfg);

        // port 1 (keyboard) interface test: 0xAB -> expect 0x00
        write_command(0xAB);
        wait_output_ready();
        if read_data() != 0x00 {
            return Err(Ps2Error::InitFailed);
        }

        // enable keyboard clock
        write_command(0xAE);

        // enable scanning (strict Set-2 policy comes in the future)
        if !send_with_ack(0xF4) {
            return Err(Ps2Error::InitFailed);
        }

        // re-enable IRQ1 in the *controller* (PIC unmask is done in chip)
        write_command(0x20);
        wait_output_ready();
        let mut cfg2 = read_data();
        cfg2 |= 1 << 0; // IRQ1 = 1 (on)
        write_command(0x60);
        write_data(cfg2);

        Ok(())
    }

    /// Legacy wrapper: keep around short-term so old call sites compile.
    /// Chip will call `init_early()` instead.
    /// for now
    #[deprecated(note = "Use init_early() from the chip bring-up.")]
    pub fn init(&self) {
        let _ = self.init_early();
    }

    /// Handle a keyboard interrupt: read a scan-code and buffer it.
    pub fn handle_interrupt(&self) {
        loop {
            if unsafe { io::inb(PS2_STATUS_PORT) } & 0x01 == 0 {
                break;
            }
            let byte = unsafe { io::inb(PS2_DATA_PORT) };
            self.push_code(byte);
            debug!("ps2 irq 0x{:02X}", byte);
        }
    }

    /// Pop the next scan-code, or None if buffer is empty.
    pub fn pop_scan_code(&self) -> Option<u8> {
        if self.count.get() == 0 {
            return None;
        }
        let t = self.tail.get();
        let b = self.buffer.borrow()[t];
        self.tail.set((t + 1) % BUFFER_SIZE);
        self.count.set(self.count.get() - 1);
        Some(b)
    }

    /// Internal: push a scan-code into the ring buffer, dropping oldest if full.
    fn push_code(&self, b: u8) {
        let h = self.head.get();
        self.buffer.borrow_mut()[h] = b;
        let nh = (h + 1) % BUFFER_SIZE;
        self.head.set(nh);

        if self.count.get() < BUFFER_SIZE {
            self.count.set(self.count.get() + 1);
        } else {
            self.tail.set((self.tail.get() + 1) % BUFFER_SIZE);
        }
    }
}

// ---------- Unit tests for ring buffer only ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let dev = Ps2Controller::new();
        // buffer empty
        assert_eq!(dev.pop_scan_code(), None);
        // simulate push via the internal helper
        dev.push_code(0x10);
        assert_eq!(dev.pop_scan_code(), Some(0x10));
        assert_eq!(dev.pop_scan_code(), None);
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let dev = Ps2Controller::new();
        // fill buffer
        for i in 0..BUFFER_SIZE {
            dev.push_code(i as u8);
        }
        // overflow one slot
        dev.push_code(0xFF);
        // should have dropped the first (0), yielding 1..BUFFER_SIZE-1 then 0xFF
        for expected in 1..BUFFER_SIZE {
            assert_eq!(dev.pop_scan_code(), Some(expected as u8));
        }
        assert_eq!(dev.pop_scan_code(), Some(0xFF));
        assert_eq!(dev.pop_scan_code(), None);
    }
}
