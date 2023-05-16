// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Test that the RNG works

use crate::tests::run_kernel_op;
use crate::PERIPHERALS;
use capsules_core::test::rng::TestEntropy32;
use kernel::debug;
use kernel::hil::entropy::Entropy32;
use kernel::static_init;

#[test_case]
fn run_csrng_entropy32() {
    debug!("check run CSRNG Entropy 32... ");
    run_kernel_op(100);

    unsafe {
        let perf = PERIPHERALS.unwrap();
        let rng = &perf.rng;

        let t = static_init!(TestEntropy32<'static>, TestEntropy32::new(rng));
        rng.set_client(t);

        #[cfg(feature = "hardware_tests")]
        t.run();
    }
    run_kernel_op(10000);
    debug!("    [ok]");
    run_kernel_op(100);
}
