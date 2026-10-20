// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Panic handler and debug output for the MPS3 board.
//!
//! Debug output is sent through UART0 using a
//! blocking byte-at-a-time write, which is safe to call even when the
//! interrupt-driven console stack is not yet initialised.

/* */
use core::fmt::Write;
use core::panic::PanicInfo;

//use kernel::debug;
use kernel::utilities::io_write::IoWrite;
use xcku::{an536_inst, uart};

struct Writer {}

static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        let mut uart = uart::ApbUart::new(an536_inst::BASE_UART2, an536_inst::FREQ_ACLK);
        uart.enable(115_200, an536_inst::FREQ_ACLK);
        uart.transmit_sync(buf);
        buf.len()
    }
}

/// Panic handler.
///
/// Prints panic information to UART2 and then halts the core.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    //use core::ptr::addr_of_mut;

    //let writer = &mut *addr_of_mut!(WRITER);

    sh::p!("##### Panic occurred: {:?} ####", pi);
    /*
        TODO need to adapt to new basee

        debug::panic_print::<_, _, _>(
            writer,
            pi,
            &cortexr52::support::nop,
            // TODO: Pass PanicResources once the console stack is wired up.
            crate::PANIC_RESOURCES.get(),
        );
    */

    // Halt the core by looping with WFI to reduce power consumption.
    loop {
        cortexr52::support::wfi();
    }
}
