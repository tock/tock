// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//! PS/2 controller – **step 3**: controller self‑test, keyboard clock enabled,
//! and IRQ1 unmasked. The interrupt handler currently reads a single byte and
//! stores it in a tiny ring buffer (used in step 4).

use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use kernel::debug;
use kernel::utilities::registers::register_bitfields;
use tock_registers::LocalRegisterCopy;
use x86::registers::io;

const PS2_DATA_PORT:   u16 = 0x60;
const PS2_STATUS_PORT: u16 = 0x64;
const PIC1_DATA_PORT:  u16 = 0x21;

const BUFFER_SIZE:   usize = 32;       // ring‑buffer depth
const TIMEOUT_LIMIT: usize = 1_000_000;

register_bitfields![u8,
    pub STATUS [
        OUTPUT_FULL OFFSET(0) NUMBITS(1), // data ready in 0x60
        INPUT_FULL  OFFSET(1) NUMBITS(1), // controller busy
    ]
];

#[inline(always)]
fn wait_input_ready() {
    let mut s = LocalRegisterCopy::<u8, STATUS::Register>::new(0);
    let mut cnt = 0;
    while {
        s.set(unsafe { io::inb(PS2_STATUS_PORT) });
        s.is_set(STATUS::INPUT_FULL)
    } {
        if { cnt += 1; cnt } >= TIMEOUT_LIMIT { break; }
    }
}

#[inline(always)]
fn wait_output_ready() {
    let mut s = LocalRegisterCopy::<u8, STATUS::Register>::new(0);
    let mut cnt = 0;
    while {
        s.set(unsafe { io::inb(PS2_STATUS_PORT) });
        !s.is_set(STATUS::OUTPUT_FULL)
    } {
        if { cnt += 1; cnt } >= TIMEOUT_LIMIT { break; }
    }
}

#[inline(always)] fn read_data() -> u8 { wait_output_ready(); unsafe { io::inb(PS2_DATA_PORT) } }
#[inline(always)] fn write_command(c: u8) { wait_input_ready(); unsafe { io::outb(PS2_STATUS_PORT, c) } }
#[inline(always)] fn write_data(d: u8)    { wait_input_ready(); unsafe { io::outb(PS2_DATA_PORT,   d) } }

// Driver Object
pub struct Ps2Controller {
    buffer: RefCell<[u8; BUFFER_SIZE]>,
    head:   Cell<usize>,
    tail:   Cell<usize>,
    _p:     PhantomData<()>,
}

impl Ps2Controller {
    pub const fn new() -> Self {
        Self { buffer: RefCell::new([0; BUFFER_SIZE]), head: Cell::new(0), tail: Cell::new(0), _p: PhantomData }
    }

    /// Initialisation for step 3.
    /// * Disable both ports, flush output buffer, run controller self‑test.
    /// * Enable keyboard clock (port 1).
    /// * Unmask IRQ 1 on the PIC so interrupts reach the kernel.
    pub fn init(&self) {
        unsafe {
            /* quiet controller */
            write_command(0xAD); // disable port 1
            write_command(0xA7); // disable port 2

            /* flush stale byte */
            while io::inb(PS2_STATUS_PORT) & 1 != 0 {
                let _ = io::inb(PS2_DATA_PORT);
            }

            /* self‑test */
            write_command(0xAA);
            let passed = { wait_output_ready(); read_data() } == 0x55;
            debug!("ps2: self‑test {}", if passed { "ok" } else { "FAIL" });

            /* enable keyboard clock */
            write_command(0xAE);

            /* unmask IRQ1 */
            let mask = io::inb(PIC1_DATA_PORT);
            io::outb(PIC1_DATA_PORT, mask & !(1 << 1));
            debug!("ps2: clock on, IRQ1 unmasked");
        }
    }

    /// Simple IRQ handler for step 3: read exactly **one** scan‑code and push
    /// it into the ring buffer (dropping if full).
    pub fn handle_interrupt(&self) {
        let byte = read_data();
        self.push_code(byte);
        debug!("ps2 irq 0x{:02X}", byte);
    }

    pub fn pop_scan_code(&self) -> Option<u8> {
        let h = self.head.get();
        let t = self.tail.get();
        if h == t { None } else {
            let b = self.buffer.borrow()[t];
            self.tail.set((t + 1) % BUFFER_SIZE);
            Some(b)
        }
    }

    /* --- internal ring‑buffer helper --- */
    fn push_code(&self, b: u8) {
        let h = self.head.get();
        let next = (h + 1) % BUFFER_SIZE;
        if next == self.tail.get() { // overwrite oldest if full
            self.tail.set((self.tail.get() + 1) % BUFFER_SIZE);
        }
        self.buffer.borrow_mut()[h] = b;
        self.head.set(next);
    }
}
