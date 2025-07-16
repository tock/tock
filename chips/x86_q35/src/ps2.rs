// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use kernel::debug;
use kernel::hil::ps2_traits::PS2Traits;
use kernel::ErrorCode;
use x86::registers::io;

/// PS/2 controller ports
const PS2_DATA_PORT: u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

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
    }
}
