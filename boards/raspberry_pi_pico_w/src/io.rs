// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright OxidOS Automotive 2025.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};

use rp2040::gpio::{GpioFunction, RPGpio, RPGpioPin};
use rp2040::uart::{UART0_BASE, Uart, UartPanicWriterConfig};

/// Default panic handler for the Raspberry Pi Pico board.
///
/// We just use the standard default provided by the debug module in the kernel.
#[cfg(not(test))]
#[panic_handler]
pub fn panic_fmt(pi: &PanicInfo) -> ! {
    // Set RX and TX pins in UART mode, in case the kernel panicked before it
    // configured the console UART.
    let gpio_tx = RPGpioPin::new(RPGpio::GPIO0);
    let gpio_rx = RPGpioPin::new(RPGpio::GPIO1);
    gpio_rx.set_function(GpioFunction::UART);
    gpio_tx.set_function(GpioFunction::UART);

    // `None` if the panic happened before this was bound (e.g. before the
    // chip's peripherals were even constructed); the panic writer leaves
    // the UART as is in that case rather than guessing a baud rate.
    let clock = raspberry_pi_pico::PANIC_CLOCKS.get().copied();

    debug::panic_print::<Uart, _, _>(
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
        raspberry_pi_pico::PANIC_RESOURCES.get(),
    );

    // Loop forever
    loop {}
}
