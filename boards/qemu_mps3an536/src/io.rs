// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Panic handler and debug output for the MPS3 board.

use core::fmt::Write;

use kernel::platform::chip::PanicWriter;
use kernel::utilities::io_write::IoWrite;
use xcku115an536::uart::ApbUart;
use xcku115an536::config;

struct PanicPrinter {
    uart: ApbUart<'static>,
}

impl IoWrite for PanicPrinter {
    fn write(&mut self, buf: &[u8]) -> usize {
        self.uart.transmit_sync(buf);
        buf.len()
    }
}

impl Write for PanicPrinter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.uart.ioprint(s);
        Ok(())
    }
}

impl PanicWriter for PanicPrinter {
    type Config = ();

    fn create_panic_writer(
        _config: Self::Config,
        _panic: &core::panic::PanicInfo,
    ) -> impl IoWrite + Write {
        let mut uart = ApbUart::new(config::BASE_UART2);
        uart.enable(115_200);
        PanicPrinter { uart }
    }
}


#[panic_handler]
pub fn panic_fmt(info: &core::panic::PanicInfo) -> ! {

    sh::p!("\n--- PANIC ---");
    sh::p!("message: {}", info.message());
    if let Some(location) = info.location() {
        sh::p!("location: {}", location);
    }

    kernel::debug::panic_print::<PanicPrinter, _, _>(
        (),
        info,
        &cortexr52::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    unsafe { 
        loop {
            core::arch::asm!("
                cpsid aif
                wfe
            ");
        }
    }
}
