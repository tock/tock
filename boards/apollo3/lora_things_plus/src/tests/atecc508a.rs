// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Test that the Atecc508a works

use crate::tests::run_kernel_op;
use crate::ATECC508A;
use kernel::debug;

#[test_case]
fn read_config() {
    run_kernel_op(100_000);

    debug!("check run ATECC508A config... ");
    run_kernel_op(100);

    unsafe {
        let atecc508a = ATECC508A.unwrap();

        atecc508a.read_config_zone().unwrap();
    }

    run_kernel_op(150_000);
    debug!("    [ok]");
    run_kernel_op(100);
}

#[test_case]
fn setup_and_lock_tock_config() {
    run_kernel_op(100_000);

    debug!("Lock the Tock config...");
    run_kernel_op(100);

    unsafe {
        let atecc508a = ATECC508A.unwrap();

        atecc508a.read_config_zone().unwrap();
        run_kernel_op(150_000);

        if atecc508a.device_locked() {
            debug!("    [ok] - Already locked");
            return;
        }

        debug!("This can not be undone!");
        run_kernel_op(100);
        debug!("Power off the board now to stop the process!");
        run_kernel_op(100);

        // Provide a chance for the user to stop the process
        run_kernel_op(1_000_000);

        debug!("Setting up config");
        atecc508a.setup_tock_config().unwrap();
        run_kernel_op(150_000);

        debug!("Locking zone config");
        atecc508a.lock_zone_config().unwrap();
        run_kernel_op(200_000);

        debug!("Generating public key");
        atecc508a.create_key_pair(0).unwrap();
        run_kernel_op(300_000);

        let public_key = atecc508a.get_public_key(0).unwrap();
        debug!("public_key: {:x?}", public_key.get());

        debug!("Locking data and OTP");
        atecc508a.lock_data_and_otp().unwrap();
        run_kernel_op(300_000);

        debug!("Locking slot 0");
        atecc508a.lock_slot0().unwrap();
        run_kernel_op(300_000);

        debug!("Reading new config");
        atecc508a.read_config_zone().unwrap();
        run_kernel_op(100_000);
    }

    run_kernel_op(100_000);
    debug!("    [ok]");
    run_kernel_op(100);
}
