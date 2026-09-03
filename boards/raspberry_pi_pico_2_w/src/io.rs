// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::fmt::Write;
use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::uart::{Configure, Parameters, Parity, StopBits, Width};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::io_write::IoWrite;

use rp2350::clocks::Clocks;
use rp2350::gpio::{GpioFunction, RPGpio, RPGpioPin};
use rp2350::uart::Uart;

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
            //set RX and TX pins in UART mode
            let gpio_tx = RPGpioPin::new(RPGpio::GPIO0);
            let gpio_rx = RPGpioPin::new(RPGpio::GPIO1);
            gpio_rx.set_function(GpioFunction::UART);
            gpio_tx.set_function(GpioFunction::UART);
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
                let clocks = &Clocks::new();
                let uart = Uart::new_uart0(clocks);
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

/// Default panic handler for the Raspberry Pi Pico 2 W board.
///
/// This board has no LED the kernel can reach on its own. GPIO 25 drives the
/// radio's chip select rather than an LED, and the LED that does exist is pin
/// 0 of the CYW43439, which needs the radio brought up before it can be lit.
/// So the panic message goes out over the console and the board halts.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    use core::ptr::addr_of_mut;
    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic_print_old(
        writer,
        pi,
        &cortexm33::support::nop,
        raspberry_pi_pico_2::PANIC_RESOURCES.get(),
    );

    // Loop forever
    loop {}
}
