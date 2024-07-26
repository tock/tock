// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright (c) 2024 Antmicro <www.antmicro.com>

use core::panic::PanicInfo;
use core::ptr::write_volatile;
use core::ptr::{addr_of, addr_of_mut};
use kernel::debug;
use veer_el2::io::Writer;

use crate::CHIP;
use crate::PROCESSES;
use crate::PROCESS_PRINTER;

static mut WRITER: Writer = Writer {};

/// Panic handler.
///
/// # Safety
/// Accesses memory-mapped registers.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic_print(
        writer,
        pi,
        &rv32i::support::nop,
        &*addr_of!(PROCESSES),
        &*addr_of!(CHIP),
        &*addr_of!(PROCESS_PRINTER),
    );

    // By writing 0xff to this address we can exit the simulation.
    // So instead of blinking in a loop let's exit the simulation.
    write_volatile(0xd0580000 as *mut u8, 0xff);

    unreachable!()
}
