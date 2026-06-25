// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;
use kernel::debug;
use kernel::hil::uart::{Parameters, Parity, StopBits, Width};
use kernel::utilities::StaticRef;
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
    debug::panic_print::<LiteXUart<crate::socc::SoCRegisterFmt>, _, _>(
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
    );

    // The system is no longer in a well-defined state; loop forever
    loop {}
}
