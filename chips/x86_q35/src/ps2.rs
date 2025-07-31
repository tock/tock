// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
<<<<<<< HEAD
// Copyright Tock Contributors 2024.
=======
// Copyright Tock Contributors 2025.
>>>>>>> ps2-incremental

use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use kernel::debug;
<<<<<<< HEAD
use kernel::hil::ps2_traits::PS2Traits;
use kernel::ErrorCode;
=======
use kernel::utilities::registers::register_bitfields;
use tock_registers::LocalRegisterCopy;
>>>>>>> ps2-incremental
use x86::registers::io;

/// PS/2 controller ports
const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
<<<<<<< HEAD

/// Status‑register bits
const STATUS_OUTPUT_FULL: u8 = 1 << 0; // data ready
const STATUS_INPUT_FULL: u8 = 1 << 1; // input buffer full

/// Timeout limit for spin loops
const TIMEOUT_LIMIT: usize = 1_000_000;

/// Depth of the scan‑code ring buffer
const BUFFER_SIZE: usize = 32;

/// PS/2 controller driver (the “8042” peripheral)
pub struct Ps2Controller<'a> {
    buffer: RefCell<[u8; BUFFER_SIZE]>,
    head: Cell<usize>,
    tail: Cell<usize>,
    _marker: PhantomData<&'a ()>,
}

impl Ps2Controller<'_> {
    /// Create a new PS/2 controller instance.
    pub fn new() -> Self {
        Ps2Controller {
            buffer: RefCell::new([0; BUFFER_SIZE]),
            head: Cell::new(0),
            tail: Cell::new(0),
            _marker: PhantomData,
        }
    }
}

impl PS2Traits for Ps2Controller<'_> {
    fn wait_input_ready() {
        let mut cnt = 0;
        while unsafe { io::inb(PS2_STATUS_PORT) } & STATUS_INPUT_FULL != 0 {
            cnt += 1;
            if cnt >= TIMEOUT_LIMIT {
                debug!("PS/2 wait_input_ready timed out");
                break;
            }
        }
    }

    fn wait_output_ready() {
        let mut cnt = 0;
        while unsafe { io::inb(PS2_STATUS_PORT) } & STATUS_OUTPUT_FULL == 0 {
            cnt += 1;
            if cnt >= TIMEOUT_LIMIT {
                debug!("PS/2 wait_output_ready timed out");
                break;
            }
        }
    }

    fn read_data() -> u8 {
        Self::wait_output_ready();
        unsafe { io::inb(PS2_DATA_PORT) }
    }

    fn write_command(cmd: u8) {
        Self::wait_input_ready();
        unsafe { io::outb(PS2_STATUS_PORT, cmd) };
    }

    fn write_data(data: u8) {
        Self::wait_input_ready();
        unsafe { io::outb(PS2_DATA_PORT, data) };
    }

    fn init(&self) {
        unsafe {
            // 1) Disable keyboard and auxiliary channels
            Self::write_command(0xAD);
            Self::write_command(0xA7);

            // 2) Flush any pending output
            while io::inb(PS2_STATUS_PORT) & STATUS_OUTPUT_FULL != 0 {
                let _ = Self::read_data();
            }

            // 3) Controller self-test (0xAA → expect 0x55)
            Self::write_command(0xAA);
            Self::wait_output_ready();
            let res = Self::read_data();
            if res != 0x55 {
                debug!("PS/2 self-test failed: {:02x}", res);
            }

            // 4) Read/modify/write config byte (enable IRQ1)
            Self::write_command(0x20);
            let mut cfg = Self::read_data();
            cfg |= 1 << 0; // enable IRQ1
            Self::write_command(0x60);
            Self::write_data(cfg);

            // 5) Test keyboard port (0xAB → expect 0x00)
            Self::write_command(0xAB);
            Self::wait_output_ready();
            let port_ok = Self::read_data();
            if port_ok != 0x00 {
                debug!("PS/2 keyboard-port test failed: {:02x}", port_ok);
            }

            // 6) Enable scanning on keyboard device (0xF4 → expect 0xFA)
            Self::write_data(0xF4);
            Self::wait_output_ready();
            let ack = Self::read_data();
            if ack != 0xFA {
                debug!("PS/2 enable-scan ACK failed: {:02x}", ack);
            }

            // 7) Re-enable keyboard channel
            Self::write_command(0xAE);

            // 8) Unmask IRQ1 on master PIC
            const PIC1_DATA: u16 = 0x21;
            let mask = io::inb(PIC1_DATA);
            io::outb(PIC1_DATA, mask & !(1 << 1));
        }
    }

    fn handle_interrupt(&self) -> Result<(), ErrorCode> {
        let sc = Self::read_data();
        self.push_code(sc)?;
        Ok(())
    }

    fn pop_scan_code(&self) -> Option<u8> {
        let head = self.head.get();
        let tail = self.tail.get();
        if head == tail {
            None
        } else {
            let byte = self.buffer.borrow()[tail];
            self.tail.set((tail + 1) % BUFFER_SIZE);
            Some(byte)
        }
    }

    fn push_code(&self, code: u8) -> Result<(), ErrorCode> {
        let head = self.head.get();
        let next = (head + 1) % BUFFER_SIZE;
        if next == self.tail.get() {
            // buffer full → drop oldest
            self.tail.set((self.tail.get() + 1) % BUFFER_SIZE);
        }
        self.buffer.borrow_mut()[head] = code;
        self.head.set(next);
        Ok(())
=======
const PIC1_DATA_PORT: u16 = 0x21;

/// Timeout limit for spin loops
const BUFFER_SIZE: usize = 32;

/// Depth of the scan-code ring buffer
const TIMEOUT_LIMIT: usize = 1_000_000;

// Define the two status‐register bits
register_bitfields![u8,
    pub STATUS [
        OUTPUT_FULL OFFSET(0) NUMBITS(1), // data ready
        INPUT_FULL  OFFSET(1) NUMBITS(1), // input buffer full
    ]
];

/// Note: There is no hardware interrupt when the input buffer empties, so we must poll bit 1.
/// See OSDev documentation:
/// https://wiki.osdev.org/I8042_PS/2_Controller#Status_Register
#[inline(always)]
fn wait_input_ready() {
    let mut s = LocalRegisterCopy::<u8, STATUS::Register>::new(0);
    let mut n = 0;
    while {
        s.set(unsafe { io::inb(PS2_STATUS_PORT) });
        s.is_set(STATUS::INPUT_FULL)
    } {
        if {
            n += 1;
            n
        } >= TIMEOUT_LIMIT
        {
            break;
        }
    }
}

/// Data-ready events trigger IRQ1, handled asynchronously in `handle_interrupt()`.
/// See OSDev documentation:
/// https://wiki.osdev.org/I8042_PS/2_Controller#Status_Register
#[inline(always)]
fn wait_output_ready() {
    let mut s = LocalRegisterCopy::<u8, STATUS::Register>::new(0);
    let mut n = 0;
    while {
        s.set(unsafe { io::inb(PS2_STATUS_PORT) });
        !s.is_set(STATUS::OUTPUT_FULL)
    } {
        if {
            n += 1;
            n
        } >= TIMEOUT_LIMIT
        {
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

/// Send a byte to the keyboard and wait for ACK (`0xFA`).
/// If the device replies RESEND (`0xFE`) we retry **once**.
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
    _p: PhantomData<()>,
}

impl Ps2Controller {
    pub const fn new() -> Self {
        Self {
            buffer: RefCell::new([0; BUFFER_SIZE]),
            head: Cell::new(0),
            tail: Cell::new(0),
            count: Cell::new(0), // ← initialize count
            _p: PhantomData,
        }
    }
    pub fn init(&self) {
        unsafe {
            /* disable both ports */
            write_command(0xAD);
            write_command(0xA7);

            /* flush any stale byte */
            while io::inb(PS2_STATUS_PORT) & 1 != 0 {
                let _ = io::inb(PS2_DATA_PORT);
            }

            /* controller self-test */
            write_command(0xAA);
            let _ = {
                wait_output_ready();
                read_data()
            } == 0x55;

            /* enable keyboard clock */
            write_command(0xAE);

            if !send_with_ack(0xF4) {
                debug!("ps2: enable-scan failed");
            }

            /* unmask IRQ 1 */
            let mask = io::inb(PIC1_DATA_PORT);
            io::outb(PIC1_DATA_PORT, mask & !(1 << 1));
            debug!("ps2: clock on, IRQ1 unmasked");
        }
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

        // track count and drop oldest if full:
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
>>>>>>> ps2-incremental
    }
}
