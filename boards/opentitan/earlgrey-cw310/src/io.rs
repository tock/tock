// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;
use earlgrey::chip_config::EarlGreyConfig;
use kernel::debug;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};
use lowrisc::uart::{Uart, UartPanicWriterConfig};

fn make_uart_config() -> UartPanicWriterConfig {
    UartPanicWriterConfig {
        registers: earlgrey::uart::UART0_BASE,
        clock_frequency: crate::ChipConfig::PERIPHERAL_FREQ,
        params: Parameters {
            baud_rate: 115200,
            stop_bits: StopBits::One,
            parity: Parity::None,
            hw_flow_control: false,
            width: Width::Eight,
        },
    }
}

#[cfg(not(test))]
use kernel::hil::gpio::Configure;
#[cfg(not(test))]
use kernel::hil::led;

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    let first_led_pin = &mut earlgrey::gpio::GpioPin::new(
        earlgrey::gpio::GPIO_BASE,
        earlgrey::pinmux::PadConfig::Output(
            earlgrey::registers::top_earlgrey::MuxedPads::Ioa6,
            earlgrey::registers::top_earlgrey::PinmuxOutsel::GpioGpio7,
        ),
        earlgrey::gpio::pins::pin7,
    );
    first_led_pin.make_output();
    let first_led = &mut led::LedLow::new(first_led_pin);

    #[cfg(feature = "sim_verilator")]
    debug::panic::<_, Uart, _, _>(
        &mut [first_led],
        make_uart_config(),
        pi,
        &|| {},
        crate::PANIC_RESOURCES.get(),
    );

    #[cfg(not(feature = "sim_verilator"))]
    debug::panic::<_, Uart, _, _>(
        &mut [first_led],
        make_uart_config(),
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    );
}

#[cfg(test)]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    #[cfg(feature = "sim_verilator")]
    debug::panic_print::<Uart, _, _>(make_uart_config(), pi, &|| {}, crate::PANIC_RESOURCES.get());
    #[cfg(not(feature = "sim_verilator"))]
    debug::panic_print::<Uart, _, _>(
        make_uart_config(),
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    // Exit QEMU with a return code of 1
    crate::tests::semihost_command_exit_failure();
}
