// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::fmt::Write;
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

    // We prefer the infinite loop to the `options(noreturn)` for `asm!` as
    // the required `isa-debug-exit` device might be missing in which case
    // the execution does not stop and generates undefined behaviour.
    let mut com1 = unsafe { BlockingSerialPort::new(COM1_BASE) };
    let _ = com1.write_fmt(format_args!(
        "BUG:  QEMU did not exit.\
        \r\n      The isa-debug-exit device is missing or is at a wrong address.\
        \r\n      Please make sure the QEMU command line uses\
        \r\n      the `-device isa-debug-exit,iobase=0xf4,iosize=0x04` argument.\
        \r\nHINT: Use `killall qemu-system-i386` or the Task Manager to stop.\
        \r\n"
    ));

    // We use the `htl` instruction in the infinite loop to prevent high CPU usage
    // if QEMU did not exit.
    loop {
        unsafe { asm!("hlt") }
    }
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
