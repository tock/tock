// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::fmt::Write;
use core::panic::PanicInfo;

/// Minimal `core::fmt::Write` straight to the SHAKTI UART registers (NOT via the
/// driver, so it can never recurse). Used by the panic handler to surface the
/// panic message — e.g. a process-fault dump from `PanicFaultPolicy` — instead
/// of just a bare marker.
struct RawUart;

impl Write for RawUart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            let tx = 0x0001_1304 as *mut u8;
            let status = 0x0001_130C as *const u16;
            for &b in s.as_bytes() {
                core::ptr::write_volatile(tx, b);
                while core::ptr::read_volatile(status) & 0x1 == 0 {}
            }
        }
        Ok(())
    }
}

/// Raw panic handler: writes the panic info directly to the SHAKTI UART (never
/// via the driver, so it cannot recurse) and ends the simulation so `app_log` is
/// flushed. A process fault under `PanicFaultPolicy` lands here, so the panic
/// message names the faulting process and its RISC-V fault state.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = write!(RawUart, "\n!PANIC! {}\n", info);
    unsafe {
        core::ptr::write_volatile(0x0002_000C as *mut u32, 1);
    }
    loop {
        core::hint::spin_loop();
    }
}
