// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};
use kernel::utilities::StaticRef;
use litex_vexriscv::led_controller::{LiteXLedController, LiteXLedRegisters};
use litex_vexriscv::uart::{LiteXUart, LiteXUartPanicWriterConfig, LiteXUartRegisters};

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    // TODO: this double-instantiates the LiteX UART. `transmit_sync` should be
    // converted into an unsafe, static method instead (which can take over UART
    // operation with the hardware in any arbitrary state, and where the caller
    // guarantees that the regular UART driver will not run following any call
    // to `transmit_sync`)

    // TODO: this double-initializes the LED controller. Similar to the UART
    // above, this should have the `panic_led` function be static and unsafe,
    // with a guarantee that the rest of the controller will not run after this
    // function is called once:
    let led0 = LiteXLedController::new(
        StaticRef::new(
            crate::socc::CSR_LEDS_BASE as *const LiteXLedRegisters<crate::socc::SoCRegisterFmt>,
        ),
        4, // 4 LEDs on this board
    );
    let panic_led = led0.panic_led(0).unwrap();

    debug::panic::<_, LiteXUart<crate::socc::SoCRegisterFmt>, _, _>(
        &mut [&panic_led],
        LiteXUartPanicWriterConfig {
            uart_base: StaticRef::new(
                crate::socc::CSR_UART_BASE
                    as *const LiteXUartRegisters<crate::socc::SoCRegisterFmt>,
            ),
            phy_args: None, // LiteX simulator has no UART phy
            params: Parameters {
                baud_rate: 115200,
                stop_bits: StopBits::One,
                parity: Parity::None,
                hw_flow_control: false,
                width: Width::Eight,
            },
        },
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    )
}
