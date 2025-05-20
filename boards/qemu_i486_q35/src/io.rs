// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::ptr;
use core::{arch::asm, panic::PanicInfo};

use kernel::debug;

use x86_q35::serial::{BlockingSerialPort, COM1_BASE};

use crate::{CHIP, PROCESSES, PROCESS_PRINTER};

/// Exists QEMU
///
/// This function requires the `-device isa-debug-exit,iobase=0xf4,iosize=0x04`
/// device enabled.
fn exit_qemu() -> ! {
    unsafe {
        asm!(
            "
        mov dx, 0xf4
        mov al, 0x01
        out dx,al
        "
        );
    }

    loop {}
}

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
unsafe fn panic_handler(pi: &PanicInfo) -> ! {
    let mut com1 = BlockingSerialPort::new(COM1_BASE);

    debug::panic_print(
        &mut com1,
        pi,
        &x86::support::nop,
        &*ptr::addr_of!(PROCESSES),
        &*ptr::addr_of!(CHIP),
        &*ptr::addr_of!(PROCESS_PRINTER),
    );

    exit_qemu();
}
