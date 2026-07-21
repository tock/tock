// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;

use kernel::debug;
use kernel::utilities::io_write::IoWrite;

/// Sim-finish MMIO register: writing 1 ends the Verilator simulation cleanly so
/// the testbench flushes `app_log`. This is the one raw write the panic path
/// keeps, since it must run after the (now standard) panic dump.
const SIM_FINISH: *mut u32 = 0x0002_000C as *mut u32;

/// Synchronous writer used by the panic handler. Drives the SHAKTI UART through
/// the chip driver's blocking `transmit_sync`, which cannot recurse into the
/// async driver path and waits for the last byte to serialize.
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
        let uart = shakti_c::uart::Uart::new(shakti_c::uart::UART0_BASE);
        uart.transmit_sync(buf);
        buf.len()
    }
}

/// Panic handler: standard `debug::panic_print_old` (banner, kernel version,
/// RISC-V CPU state, per-process fault dump), then end the Verilator sim (no
/// LEDs on this SoC) so `app_log` is flushed.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic_print_old(
        writer,
        pi,
        &rv64i::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    core::ptr::write_volatile(SIM_FINISH, 1);

    loop {
        core::hint::spin_loop();
    }
}
