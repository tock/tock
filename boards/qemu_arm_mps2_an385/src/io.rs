// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::panic::PanicInfo;

use kernel::debug;
use kernel::debug::PanicResources;
use kernel::hil::uart;
use kernel::utilities::single_thread_value::SingleThreadValue;

/// Board-owned panic-time resources.
///
/// This can't live in `qemu_arm_mps2_lib` since a `static` can't be generic
/// over the `CortexMVariant` the way `qemu_arm_mps2_lib::start()` is.
pub(crate) static PANIC_RESOURCES: SingleThreadValue<
    PanicResources<
        qemu_arm_mps2_lib::ChipHw<cortexm3::CortexM3>,
        qemu_arm_mps2_lib::ProcessPrinterInUse,
    >,
> = SingleThreadValue::new();

/// Panic handler.
#[panic_handler]
pub unsafe fn panic_fmt(info: &PanicInfo) -> ! {
    debug::panic_print::<qemu_arm_mps2_chip::uart::UartPanicWriter, _, _>(
        qemu_arm_mps2_chip::uart::UartPanicWriterConfig {
            base: qemu_arm_mps2_chip::uart::UART0_BASE,
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
        PANIC_RESOURCES.get(),
    );

    // SAFETY: the system is no longer in a well-defined state (we're in the
    // panic handler), so falling through if there's no semihosting host to
    // service this (e.g. real hardware, or QEMU without `-semihosting`) is
    // fine -- we don't resume normal execution either way, per the loop
    // below.
    unsafe {
        cortexm3::support::semihost_terminate();
    }

    loop {}
}
