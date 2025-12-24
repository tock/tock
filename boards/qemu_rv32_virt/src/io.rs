// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::uart;

/// Panic handler.
#[cfg(not(test))]
#[panic_handler]
pub unsafe fn panic_fmt(pi: &PanicInfo) -> ! {
    debug::panic_print::<qemu_rv32_virt_chip::uart::Uart16550, _, _>(
        qemu_rv32_virt_chip::uart::UartPanicWriterConfig {
            params: uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            },
        },
        pi,
        &rv32i::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    // The system is no longer in a well-defined state. Use
    // semihosting commands to exit QEMU with a return code of 1.
    rv32i::semihost_command(0x18, 1, 0);

    // To satisfy the ! return type constraints.
    loop {}
}
