// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;

use kernel::debug::{self, IoWrite};
use kernel::utilities::registers::interfaces::Readable;

use crate::{CHIP, ThreadType};

struct Writer {}

static mut WRITER: Writer = Writer {};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        let uart = qemu_rv32_virt_chip::uart::Uart16550::new(qemu_rv32_virt_chip::uart::UART0_BASE);
        uart.transmit_sync(buf);
        buf.len()
    }
}

/// Panic handler.
// #[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut *core::ptr::addr_of_mut!(WRITER);
    let thread_type = rv32i::csr::CSR.mhartid.extract().get().try_into().expect("Invalid thread type");

    let chip: &Option<&_> = (*core::ptr::addr_of_mut!(CHIP))
        .get_mut()
        .expect("This thread cannot access thread-local chip construct")
        .enter_nonreentrant(|chip| {
            // Escape the current scope to get a static reference as required by
            // `debug::panic_print()`.
            &*(chip as *mut _)
        });

    match thread_type {
        ThreadType::Main => {
            debug::panic_print::<_, _, _>(
                writer,
                pi,
                &rv32i::support::nop,
                &*core::ptr::addr_of!(crate::threads::main_thread::PROCESSES),
                &chip,
                &*core::ptr::addr_of!(crate::threads::main_thread::PROCESS_PRINTER),
            );
        }
        ThreadType::Application => {
            debug::panic_print::<_, _, _>(
                writer,
                pi,
                &rv32i::support::nop,
                &*core::ptr::addr_of!(crate::threads::app_thread::PROCESSES),
                &chip,
                &*core::ptr::addr_of!(crate::threads::app_thread::PROCESS_PRINTER),
            );
        }
    }

    // The system is no longer in a well-defined state. Use
    // semihosting commands to exit QEMU with a return code of 1.
    rv32i::semihost_command(0x18, 1, 0);

    // To satisfy the ! return type constraints.
    loop {}
}
