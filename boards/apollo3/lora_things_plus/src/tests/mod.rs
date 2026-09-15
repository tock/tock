// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::BOARD;
use crate::NUM_PROCS;
use crate::PANIC_RESOURCES;
use crate::PLATFORM;
use kernel::capabilities;
use kernel::create_capability;
use kernel::debug;

fn run_kernel_op(loops: usize) {
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

    let board = *BOARD.get().unwrap();
    let platform = *PLATFORM.get().unwrap();

    for _i in 0..loops {
        board.kernel_loop_operation(
            platform,
            PANIC_RESOURCES.get().and_then(|pr| pr.chip.get()).unwrap(),
            None::<&kernel::ipc::IPC<{ NUM_PROCS as u8 }>>,
            true,
            &main_loop_cap,
        );
    }
}

#[test_case]
fn trivial_assertion() {
    debug!("trivial assertion... ");
    run_kernel_op(10000);

    assert_eq!(1, 1);

    debug!("    [ok]");
    run_kernel_op(10000);
}

#[cfg(feature = "atecc508a")]
mod atecc508a;
#[cfg(feature = "atecc508a")]
mod csrng;
#[cfg(feature = "atecc508a")]
mod sha;

mod environmental_sensors;
mod multi_alarm;
mod spi_controller;
