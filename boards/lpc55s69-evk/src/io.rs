// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Board‑level I/O and panic infrastructure for the LPC55S69‑EVK.
//!
//! This module provides:
//! - A `Writer` type that implements both `core::fmt::Write` and
//!   Tock’s `IoWrite` trait, allowing formatted output and debug
//!   messages to be sent over UART0.
//! - UART initialization and pin configuration via the IOCON block
//!   (TX on P0_29, RX on P0_30).
//! - A global `WRITER` instance used by the kernel’s debug system.
//! - A `panic_handler` that configures an LED (P1_6) as a panic
//!   indicator and routes panic output through the UART writer.
//!
//! Together, these components provide console output and visual
//! feedback during normal operation and in panic situations.
//!
//! Reference: *LPC55S6x/LPC55S2x/LPC552x User Manual* (NXP).

use crate::LPCPin;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
use kernel::debug::{self, IoWrite};
use kernel::hil::gpio::Configure;
use kernel::hil::led::LedHigh;
use kernel::hil::uart::{Configure as UARTconfig, Parameters, Parity, StopBits, Width};
use kernel::utilities::cells::OptionalCell;
use lpc55s6x::gpio::GpioPin;
use lpc55s6x::iocon::{Config, Function, Iocon, Pull, Slew};
use lpc55s6x::uart::Uart;

pub struct Writer {
    uart: OptionalCell<&'static Uart<'static>>,
    rtt: OptionalCell<&'static segger::rtt::SeggerRttMemory<'static>>,
}

impl Writer {
    pub fn set_uart(&self, uart: &'static Uart) {
        self.uart.set(uart);
    }

    pub fn set_rtt_memory(&self, rtt: &'static segger::rtt::SeggerRttMemory<'static>) {
        self.rtt.set(rtt);
    }

    fn configure_uart(&self, uart: &Uart) {
        if !uart.is_configured() {
            let params = Parameters {
                // USART initial configuration, using default settings
                baud_rate: 115200,
                width: Width::Eight,
                stop_bits: StopBits::One,
                parity: Parity::None,
                hw_flow_control: false,
            };

            let _ = uart.configure(params);

            let iocon = Iocon::new();

            iocon.configure_pin(
                LPCPin::P0_29,
                Config {
                    function: Function::Alt1,
                    pull: Pull::None,
                    digital_mode: true,
                    slew: Slew::Standard,
                    invert: false,
                    open_drain: false,
                },
            );
            iocon.configure_pin(
                LPCPin::P0_30,
                Config {
                    function: Function::Alt1,
                    pull: Pull::None,
                    digital_mode: true,
                    slew: Slew::Standard,
                    invert: false,
                    open_drain: false,
                },
            );
        }
    }

    fn write_to_uart(&self, uart: &Uart, buf: &[u8]) {
        for &c in buf {
            uart.send_byte(c);
            while !uart.uart_is_writable() {}
        }
    }
}

pub static mut WRITER: Writer = Writer {
    uart: OptionalCell::empty(),
    rtt: OptionalCell::empty(),
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
                let uart = Uart::new_uart0();
                self.configure_uart(&uart);
                self.write_to_uart(&uart, buf);
            },
            |uart| {
                self.configure_uart(uart);
                self.write_to_uart(uart, buf);
            },
        );
        self.rtt.map(|rtt| {
            rtt.write_sync(buf);
        });
        buf.len()
    }
}

#[panic_handler]
pub unsafe fn panic_fmt(panic_info: &PanicInfo) -> ! {
    let iocon_ctrl = Iocon::new();
    let led_pin_config = Config {
        function: Function::GPIO,
        pull: Pull::Up,
        slew: Slew::Standard,
        invert: false,
        digital_mode: true,
        open_drain: false,
    };
    iocon_ctrl.configure_pin(LPCPin::P1_6, led_pin_config);
    let red_led = GpioPin::new(LPCPin::P1_6);
    red_led.make_output();
    let led = &mut LedHigh::new(&red_led);
    let writer = &mut *addr_of_mut!(WRITER);

    debug::panic(
        &mut [led],
        writer,
        panic_info,
        &cortexm33::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
