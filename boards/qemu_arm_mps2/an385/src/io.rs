// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::hil::uart;

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    debug::panic_print::<qemu_arm_mps2::uart::UartPanicWriter, _, _>(
        qemu_arm_mps2::uart::UartPanicWriterConfig {
            base: qemu_arm_mps2::uart::UART0_BASE,
            params: uart::Parameters {
                baud_rate: 115200,
                stop_bits: uart::StopBits::One,
                parity: uart::Parity::None,
                hw_flow_control: false,
                width: uart::Width::Eight,
            },
        },
        info,
        &cortexm3::support::nop,
        crate::PANIC_RESOURCES.get(),
    );

    // SAFETY: the system is no longer in a well-defined state (we're in the
    // panic handler), so falling through if there's no semihosting host to
    // service this (e.g. real hardware, or QEMU without `-semihosting`) is
    // fine -- we don't resume normal execution either way, per the loop
    // below.
    unsafe {
        use cortexm3::semihosting;
        semihosting::terminate(semihosting::SysexitReason::ADP_Stopped_RunTimeErrorUnknown);
    }

    loop {}
}
