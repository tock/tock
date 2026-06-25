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

use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::gpio::Configure;
use kernel::hil::led::LedHigh;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};
use lpc55s6x::gpio::{GpioPin, LPCPin};
use lpc55s6x::iocon::{Config, Function, Iocon, Pull, Slew};

use lpc55s6x::uart::{Uart, UartPanicWriterConfig};

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

    if crate::USB_DEBUGGING {
        // Use the RTT output that needs to be setup in main.rs.

        crate::RTT_BUFFER.get().map_or_else(
            || debug::panic_blink_forever(&mut [led]),
            |rtt| {
                debug::panic::<_, segger::rtt::SeggerRttMemory, _, _>(
                    &mut [led],
                    rtt,
                    panic_info,
                    &cortexm33::support::nop,
                    crate::PANIC_RESOURCES.get(),
                )
            },
        )
    } else {
        // Use the LPC55 UART for panic output.

        debug::panic::<_, Uart, _, _>(
            &mut [led],
            UartPanicWriterConfig {
                params: Parameters {
                    baud_rate: 115200,
                    stop_bits: StopBits::One,
                    parity: Parity::None,
                    hw_flow_control: false,
                    width: Width::Eight,
                },
                id: lpc55s6x::uart::UartId::Uart0,
                clocks: &lpc55s6x::clocks::Clock::new(),
                flexcomm: &lpc55s6x::flexcomm::Flexcomm::new(0),
                iocon: &iocon_ctrl,
                pin1: LPCPin::P0_29,
                pin2: LPCPin::P0_30,
            },
            panic_info,
            &cortexm33::support::nop,
            crate::PANIC_RESOURCES.get(),
        )
    }
}
