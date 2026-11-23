// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::led::LedHigh;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};

use rp2040::gpio::{GpioFunction, RPGpio, RPGpioPin};
use rp2040::uart::{UART0_BASE, Uart, UartPanicWriterConfig};

/// Default panic handler for the pico explorer base board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[panic_handler]
pub fn panic_fmt(pi: &PanicInfo) -> ! {
    // LED is connected to GPIO 25
    let led_kernel_pin = &RPGpioPin::new(RPGpio::GPIO25);
    let led = &mut LedHigh::new(led_kernel_pin);

    // Set RX and TX pins in UART mode, in case the kernel panicked before it
    // configured the console UART.
    let gpio_tx = RPGpioPin::new(RPGpio::GPIO0);
    let gpio_rx = RPGpioPin::new(RPGpio::GPIO1);
    gpio_rx.set_function(GpioFunction::UART);
    gpio_tx.set_function(GpioFunction::UART);

    // `None` if the panic happened before this was bound (e.g. before the
    // chip's peripherals were even constructed); the panic writer leaves
    // the UART as is in that case rather than guessing a baud rate.
    let clock = crate::PANIC_CLOCKS.get().copied();

    debug::panic::<_, Uart, _, _>(
        &mut [led],
        UartPanicWriterConfig {
            registers: UART0_BASE,
            clock,
            params: Parameters {
                baud_rate: 115200,
                width: Width::Eight,
                parity: Parity::None,
                stop_bits: StopBits::One,
                hw_flow_control: false,
            },
        },
        pi,
        &cortexm0p::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
