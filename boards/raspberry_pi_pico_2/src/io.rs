// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use core::fmt::Write;
use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led::LedHigh;
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

/// Default panic handler for the Raspberry Pi Pico 2 board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // LED is connected to GPIO 25

    use core::ptr::addr_of_mut;
    let led_kernel_pin = &RPGpioPin::new(RPGpio::GPIO25);
    let led = &mut LedHigh::new(led_kernel_pin);
    let writer = &mut *addr_of_mut!(WRITER);

    // SCRATCH DIAGNOSTIC (fail-stop / gate-round debugging): print core 1's
    // round-loop state before the standard panic dump. Reliable to print
    // here even from core 1 -- whichever core panics first is, by
    // definition, the one still actively driving execution; its peer is
    // blocked spin-waiting for a reply that will never come, so there's no
    // concurrent UART writer to race against (see the interleaving bug this
    // avoided in an earlier attempt at raw core-1 UART writes).
    let _ = write!(
        writer,
        "\r\n[panic diag] core1_stage={} core1_round={} core1_l1_events={}\r\n",
        crate::CORE1_STAGE.load(core::sync::atomic::Ordering::Relaxed),
        crate::CORE1_ROUND.load(core::sync::atomic::Ordering::Relaxed),
        crate::CORE1_L1_EVENT_COUNT.load(core::sync::atomic::Ordering::Relaxed),
    );

    debug::panic_old(
        &mut [led],
        writer,
        pi,
        &cortexm33::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
