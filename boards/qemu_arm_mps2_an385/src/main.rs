// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Tock kernel for the QEMU ARM MPS2 AN385 (Cortex-M3) machine.
//!
//! This is a purely virtual platform: ARM's own CMSDK reference design, as
//! emulated by QEMU, not a real vendor chip. See `chips/qemu_arm_mps2_chip`
//! for the peripheral drivers and `README.md` for what is and is not
//! emulated (notably: GPIO pin state is not observable under this QEMU
//! machine, so this board does not expose a GPIO capsule; LEDs are
//! implemented against the separate, genuinely-emulated FPGAIO block).
//!
//! This board and `qemu_arm_mps2_an386` are identical other than their CPU
//! core; all the shared setup lives in `qemu_arm_mps2_lib`.

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::create_capability;
use kernel::static_init;

pub mod io;

kernel::stack_size! {0x2000}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    // SAFETY: `main` is only ever invoked once, by the reset handler, before
    // anything else touches the chip's peripherals or kernel state -- see
    // `qemu_arm_mps2_lib::early_init()`'s safety doc. `CortexM3` is this
    // board's actual CPU core.
    let early =
        unsafe { qemu_arm_mps2_lib::early_init::<cortexm3::CortexM3>(&io::PANIC_RESOURCES) };

    // Must be allocated here, not inside `qemu_arm_mps2_lib`: a `static`
    // can't reference a generic function's own type parameter, so this one
    // `static_init!()` needs a concrete, non-generic call site. See
    // `qemu_arm_mps2_lib::early_init()`'s docs.
    let chip = static_init!(
        qemu_arm_mps2_lib::ChipHw<cortexm3::CortexM3>,
        qemu_arm_mps2_lib::ChipHw::<cortexm3::CortexM3>::new(early.peripherals)
    );

    // SAFETY: called immediately after the `early_init()`/`static_init!()`
    // pair above, from the same boot, same `C` -- see
    // `qemu_arm_mps2_lib::finish_start()`'s safety doc.
    let (board_kernel, platform, chip) = unsafe { qemu_arm_mps2_lib::finish_start(early, chip) };

    kernel::debug!("QEMU MPS2 AN385 (Cortex-M3) initialization complete.");
    kernel::debug!("Entering main loop.");

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop::<
        qemu_arm_mps2_lib::Platform,
        qemu_arm_mps2_lib::ChipHw<cortexm3::CortexM3>,
        { qemu_arm_mps2_lib::NUM_PROCS as u8 },
    >(platform, chip, None, &main_loop_capability);
}
