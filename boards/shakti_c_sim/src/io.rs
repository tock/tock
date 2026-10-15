// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::uart;

/// Sim-finish MMIO register: writing 1 ends the Verilator simulation cleanly so
/// the testbench flushes its captured UART log.
const SIM_FINISH: *mut u32 = 0x0002_000C as *mut u32;

/// Panic handler.
///
/// Uses the standard `debug::panic_print` interface with the chip's synchronous
/// `UartPanicWriter` (no `static mut`): prints the panic banner, kernel version,
/// RISC-V CPU state, and per-process fault info. This board has no LEDs, so
/// instead of blinking it ends the Verilator simulation.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    debug::panic_print::<shakti_c::uart::UartPanicWriter, _, _>(
        shakti_c::uart::UartPanicWriterConfig {
            params: uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            },
        },
        pi,
        &rv64i::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    core::ptr::write_volatile(SIM_FINISH, 1);
    loop {
        core::hint::spin_loop();
    }
}
