// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::panic::PanicInfo;
use core::ptr;

use kernel::debug;

use pc::serial::{BlockingSerialPort, COM1_BASE};

use crate::{CHIP, PROCESSES, PROCESS_PRINTER};

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
unsafe fn panic_handler(pi: &PanicInfo) -> ! {
    let mut com1 = BlockingSerialPort::new(COM1_BASE);

    debug::panic_print(
        &mut com1,
        pi,
        &tock_x86::support::nop,
        &*ptr::addr_of!(PROCESSES),
        &*ptr::addr_of!(CHIP),
        &*ptr::addr_of!(PROCESS_PRINTER),
    );

    loop {}
}
