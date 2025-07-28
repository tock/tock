// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//! PS/2 controller – step 1 run the controller self‑test only.  No clock, no IRQ.

use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use kernel::debug;
use kernel::utilities::registers::register_bitfields;
use tock_registers::LocalRegisterCopy;
use x86::registers::io;

const PS2_DATA_PORT:   u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;

const BUFFER_SIZE:   usize = 32;
const TIMEOUT_LIMIT: usize = 1_000_000;

register_bitfields![u8,
    pub STATUS [
        OUTPUT_FULL OFFSET(0) NUMBITS(1), // data available in 0x60
        INPUT_FULL  OFFSET(1) NUMBITS(1), // controller busy
    ]
];

#[inline(always)]
fn wait_input_ready() {
    let mut status = LocalRegisterCopy::<u8, STATUS::Register>::new(0);
    let mut cnt = 0;
    while {
        status.set(unsafe { io::inb(PS2_STATUS_PORT) });
        status.is_set(STATUS::INPUT_FULL)
    } {
        cnt += 1;
        if cnt >= TIMEOUT_LIMIT {
            debug!("ps2: wait_input_ready timeout");
            break;
        }
    }
}

#[inline(always)]
fn wait_output_ready() {
    let mut status = LocalRegisterCopy::<u8, STATUS::Register>::new(0);
    let mut cnt = 0;
    while {
        status.set(unsafe { io::inb(PS2_STATUS_PORT) });
        !status.is_set(STATUS::OUTPUT_FULL)
    } {
        cnt += 1;
        if cnt >= TIMEOUT_LIMIT {
            debug!("ps2: wait_output_ready timeout");
            break;
        }
    }
}

#[inline(always)]
fn read_data() -> u8 {
    wait_output_ready();
    unsafe { io::inb(PS2_DATA_PORT) }
}

#[inline(always)]
fn write_command(cmd: u8) {
    wait_input_ready();
    unsafe { io::outb(PS2_STATUS_PORT, cmd) };
}

#[inline(always)]
fn write_data(data: u8) {
    wait_input_ready();
    unsafe { io::outb(PS2_DATA_PORT, data) };
}


/// Driver object


/// IRQs are still masked; buffer is unused until step #
pub struct Ps2Controller {
    buffer:   RefCell<[u8; BUFFER_SIZE]>,
    head:     Cell<usize>,
    tail:     Cell<usize>,
    _phantom: PhantomData<()>,
}

impl Ps2Controller {
    pub const fn new() -> Self {
        Self {
            buffer:   RefCell::new([0; BUFFER_SIZE]),
            head:     Cell::new(0),
            tail:     Cell::new(0),
            _phantom: PhantomData,
        }
    }

    /// Step 1 initialisation – only run the built‑in self‑test.
    /// No device clock and no PIC mask changes yet.
    pub fn init(&self) {
        unsafe {
            // Disable both channels (keyboard & aux) so they stay quiet.
            write_command(0xAD); // disable port 1
            write_command(0xA7); // disable port 2

            // Flush any residual byte in the output buffer.
            while unsafe { io::inb(PS2_STATUS_PORT) } & 0x01 != 0 {
                let _ = unsafe { io::inb(PS2_DATA_PORT) };
            }

            // Controller self‑test (0xAA => 0x55 on success).
            write_command(0xAA);
            wait_output_ready();
            let result = read_data();
            if result == 0x55 {
                debug!("ps2: self‑test passed");
            } else {
                debug!("ps2: self‑test FAILED (0x{:02X})", result);
            }
        }
    }

    /* --- stubs for later steps --- */

    pub fn handle_interrupt(&self) {
        // will be filled in Step # when IRQ1 is unmasked
    }

    pub fn pop_scan_code(&self) -> Option<u8> {
        None // buffer unused until Step #
    }
}
