// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use kernel::debug;
use kernel::utilities::io_write::IoWrite;

struct Writer {
    uart: litex_vexriscv::uart::LiteXUart<'static, crate::socc::SoCRegisterFmt>,
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        self.uart.transmit_sync(buf);
        buf.len()
    }
}

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // TODO: this double-instantiates the LiteX UART. `transmit_sync` should be
    // converted into an unsafe, static method instead (which can take over UART
    // operation with the hardware in any arbitrary state, and where the caller
    // guarantees that the regular UART driver will not run following any call
    // to `transmit_sync`)
    let mut writer = Writer {
        uart: litex_vexriscv::uart::LiteXUart::new(
            kernel::utilities::StaticRef::new(
                crate::socc::CSR_UART_BASE
                    as *const litex_vexriscv::uart::LiteXUartRegisters<crate::socc::SoCRegisterFmt>,
            ),
            None, // LiteX simulator has no UART phy
        ),
    };

    debug::panic_print(
        &mut writer,
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    // The system is no longer in a well-defined state; loop forever
    loop {}
}
