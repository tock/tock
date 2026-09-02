// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Tock kernel for the QEMU ARM MPS2 AN385 (Cortex-M3) machine.
//!
//! The shared setup lives in `mps2_base`.

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::create_capability;
use kernel::debug::PanicResources;
use kernel::static_init;
use kernel::utilities::single_thread_value::SingleThreadValue;

pub mod io;

kernel::stack_size! {0x2000}

/// Board-owned panic-time resources, populated by `mps2_base` during boot
/// and read back by the `#[panic_handler]` in `io.rs`.
static PANIC_RESOURCES: SingleThreadValue<
    PanicResources<
        mps2_base::ChipHw<qemu_arm_mps2_an385::CortexM3>,
        mps2_base::ProcessPrinterInUse,
    >,
> = SingleThreadValue::new();

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    // SAFETY: `main` is only ever invoked once, by the reset handler, before
    // anything else touches the chip's peripherals or kernel state -- see
    // `mps2_base::start()`'s safety doc. `CortexM3` is this board's actual CPU
    // core.
    let (board_kernel, platform, chip) = unsafe {
        mps2_base::start::<qemu_arm_mps2_an385::CortexM3, _>(&PANIC_RESOURCES, |peripherals| {
            // The chip instance names the Cortex-M variant concretely, which
            // `static_init!()` cannot do inside a generic function.
            static_init!(
                mps2_base::ChipHw<qemu_arm_mps2_an385::CortexM3>,
                mps2_base::ChipHw::<qemu_arm_mps2_an385::CortexM3>::new(peripherals)
            )
        })
    };

    kernel::debug!("QEMU MPS2 AN385 (Cortex-M3) initialization complete.");
    kernel::debug!("Entering main loop.");

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop::<
        mps2_base::Platform,
        mps2_base::ChipHw<qemu_arm_mps2_an385::CortexM3>,
        { mps2_base::NUM_PROCS as u8 },
    >(platform, chip, None, &main_loop_capability);
}
