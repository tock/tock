// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Module for interacting with legacy 8259 PIC
//!
//! This implementation is based on guidance from the following sources:
//!
//! * <https://wiki.osdev.org/8259_PIC>
//! * <https://github.com/rust-osdev/pic8259>

use x86::registers::io;
use x86::IDT_RESERVED_EXCEPTIONS;

/// PIC initialization command
const PIC_CMD_INIT: u8 = 0x10;

/// Flag that can be added to `PIC_CMD_INIT` indicating that a fourth configuration word will _not_
/// be provided.
const PIC_CMD_INIT_NO_ICW4: u8 = 0x01;

/// PIC end-of-interrupt command
const PIC_CMD_EOI: u8 = 0x20;

/// Tells PIC to operate in 8086 mode
const PIC_MODE_8086: u8 = 0x01;

/// I/O port address of PIC 1 command register
const PIC1_CMD: u16 = 0x20;

/// I/O port address of PIC 1 data register
const PIC1_DATA: u16 = PIC1_CMD + 1;

/// Offset to which PIC 1 interrupts are re-mapped
pub(crate) const PIC1_OFFSET: u8 = IDT_RESERVED_EXCEPTIONS;

/// Number of interrupts handled by PIC 1
const PIC1_NUM_INTERRUPTS: u8 = 8; // IRQ 0-7

/// I/O port address of PIC 2 command register
const PIC2_CMD: u16 = 0xa0;

/// I/O port address of PIC 2 data register
const PIC2_DATA: u16 = PIC2_CMD + 1;

/// Offset to which PIC 2 interrupts are re-mapped
pub(crate) const PIC2_OFFSET: u8 = PIC1_OFFSET + PIC1_NUM_INTERRUPTS;

/// I/O dummy port used for introducing a delay between PIC commands
const POST_PORT: u16 = 0x80;

/// Initializes the system's primary and secondary PICs in a chained configuration and unmasks all
/// interrupts.
///
/// ## Safety
///
/// Calling this function will cause interrupts to start firing. This means the IDT must already be
/// initialized with valid handlers for all possible interrupt numbers (i.e. by a call to
/// [`handlers::init`][super::handlers::init]).
pub(crate) unsafe fn init() {
    unsafe {
        let wait = || io::outb(POST_PORT, 0);

        // Begin initialization
        io::outb(PIC1_CMD, PIC_CMD_INIT | PIC_CMD_INIT_NO_ICW4);
        wait();
        io::outb(PIC2_CMD, PIC_CMD_INIT | PIC_CMD_INIT_NO_ICW4);
        wait();

        // Re-map interrupt offsets
        io::outb(PIC1_DATA, PIC1_OFFSET);
        wait();
        io::outb(PIC2_DATA, PIC2_OFFSET);
        wait();

        // Configure chaining
        io::outb(PIC1_DATA, 4);
        wait();
        io::outb(PIC2_DATA, 2);
        wait();

        // Configure mode
        io::outb(PIC1_DATA, PIC_MODE_8086);
        wait();
        io::outb(PIC2_DATA, PIC_MODE_8086);
        wait();

        // Unmask all interrupts
        io::outb(PIC1_DATA, 0x00);
        io::outb(PIC2_DATA, 0x00);
    }
}

/// Sends an end-of-interrupt signal to the PIC responsible for generating interrupt `num`.
///
/// If `num` does not correspond to either the primary or secondary PIC, then no action is taken.
///
/// ## Safety
///
/// This function must _only_ be called from an interrupt servicing routine. Calling this function
/// from the normal kernel loop could interfere with this crate's interrupt handling logic.
pub(crate) unsafe fn eoi(num: u32) {
    let _ = u8::try_from(num).map(|num| unsafe {
        if (PIC1_OFFSET..PIC1_OFFSET + 8).contains(&num) {
            io::outb(PIC1_CMD, PIC_CMD_EOI);
        } else if (PIC2_OFFSET..PIC2_OFFSET + 8).contains(&num) {
            io::outb(PIC2_CMD, PIC_CMD_EOI);
        }
    });
}
