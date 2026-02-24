// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use core::fmt::Write;
use core::panic::PanicInfo;

use kernel::debug::{self, IoWrite};
use kernel::hil::uart::{Configure, Parameters, Parity, StopBits, Width};
use kernel::utilities::cells::OptionalCell;

use musca_b1::uart::Uart;

/// Writer is used by kernel::debug to panic message to the serial port.
pub struct Writer {
    uart: OptionalCell<&'static Uart<'static>>,
}

impl Writer {
    pub fn set_uart(&self, uart: &'static Uart) {
        self.uart.set(uart);
    }

    fn configure_uart(&self, uart: &Uart) {
        if !uart.is_configured() {
            let parameters = Parameters {
                baud_rate: 115200,
                width: Width::Eight,
                parity: Parity::None,
                stop_bits: StopBits::One,
                hw_flow_control: false,
            };
            //configure parameters of uart for sending bytes
            let _ = uart.configure(parameters);
        }
    }

    fn write_to_uart(&self, uart: &Uart, buf: &[u8]) {
        for &c in buf {
            uart.send_byte(c);
        }
    }
}

/// Global static for debug writer
pub static mut WRITER: Writer = Writer {
    uart: OptionalCell::empty(),
};

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) -> usize {
        self.uart.map_or_else(
            || {
                let uart = Uart::new_uart0_sec();
                self.configure_uart(&uart);
                self.write_to_uart(&uart, buf);
            },
            |uart| {
                self.configure_uart(uart);
                self.write_to_uart(uart, buf);
            },
        );
        buf.len()
    }
}

#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    use core::ptr::addr_of_mut;
    let writer = &mut *addr_of_mut!(WRITER);

    let _ = writer.write_str("Panic: ");

    debug::panic_print::<_, _, _>(
        writer,
        pi,
        &cortexm33::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    loop {}
}
