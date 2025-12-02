// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::BOARD;
use crate::CHIP;
use crate::MAIN_CAP;
use crate::PLATFORM;
use kernel::debug;

pub fn semihost_command_exit_success() -> ! {
    run_kernel_op(10000);

    // Exit QEMU with a return code of 0
    unsafe {
        rv32i::semihost_command(0x18, 0x20026, 0);
    }
    loop {}
}

pub fn semihost_command_exit_failure() -> ! {
    run_kernel_op(10000);

    // Exit QEMU with a return code of 1
    unsafe {
        rv32i::semihost_command(0x18, 1, 0);
    }
    loop {}
}

fn run_kernel_op(loops: usize) {
    unsafe {
        for _i in 0..loops {
            BOARD.unwrap().kernel_loop_operation(
                PLATFORM.unwrap(),
                CHIP.unwrap(),
                None::<&kernel::ipc::IPC<0>>,
                true,
                MAIN_CAP.unwrap(),
            );
        }
    }
}

#[test_case]
fn trivial_assertion() {
    debug!("trivial assertion... ");
    run_kernel_op(100);

    assert_eq!(1, 1);

    debug!("    [ok]");
    run_kernel_op(100);
}

mod aes_test;
mod csrng;
mod flash;
mod hmac;
mod multi_alarm;
mod otbn;
mod rsa;
mod rsa_4096;
mod sha256soft_test; // Test software SHA capsule
mod sip_hash;
mod spi_host;
