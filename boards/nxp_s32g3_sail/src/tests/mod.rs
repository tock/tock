// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Deterministic hardware tests for the NXP S32G3 SAIL board.
//!
//! Enable them with the `test-harness` feature. The launcher owns suite
//! ordering; each suite reports completion through Tock's capsule-test API.

use core::cell::Cell;

use capsules_core::test::capsule_test::{CapsuleTestClient, CapsuleTestError};
use capsules_core::virtualizers::virtual_alarm::MuxAlarm;
use kernel::static_init;
use nxp_s32g3::linflexd::LinFlexD;
use nxp_s32g3::stm::Stm;

pub(crate) mod stm_suite;
pub(crate) mod uart_suite;

struct TestLauncher {
    test_index: Cell<usize>,
    mux_alarm: &'static MuxAlarm<'static, Stm<'static>>,
    stm: &'static Stm<'static>,
    lf1: &'static LinFlexD<'static>,
}

impl TestLauncher {
    const fn new(
        mux_alarm: &'static MuxAlarm<'static, Stm<'static>>,
        stm: &'static Stm<'static>,
        lf1: &'static LinFlexD<'static>,
    ) -> Self {
        Self {
            test_index: Cell::new(0),
            mux_alarm,
            stm,
            lf1,
        }
    }

    fn next(&'static self) {
        let test_index = self.test_index.get();
        self.test_index.set(test_index + 1);
        match test_index {
            0 => unsafe { stm_suite::run(self.mux_alarm, self.stm, self) },
            1 => unsafe { uart_suite::run(self.lf1, self) },
            _ => panic!("S32G3 test harness complete"),
        }
    }
}

impl CapsuleTestClient for TestLauncher {
    fn done(&'static self, _result: Result<(), CapsuleTestError>) {
        self.next();
    }
}

/// Start all board hardware tests in their declared order.
///
/// # Safety
/// Must be called exactly once after the STM mux and LF1 are configured.
pub unsafe fn run(
    mux_alarm: &'static MuxAlarm<'static, Stm<'static>>,
    stm: &'static Stm<'static>,
    lf1: &'static LinFlexD<'static>,
) {
    let launcher = static_init!(TestLauncher, TestLauncher::new(mux_alarm, stm, lf1));
    launcher.next();
}
