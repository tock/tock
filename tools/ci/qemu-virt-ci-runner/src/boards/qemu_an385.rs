// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use std::time::Duration;

use crate::{App, TestCase, TestStep};

pub static BOARD: super::Board = super::Board {
    name: "an385",
    board_dir: "../../../boards/qemu_arm_mps2/an385",
    tock_targets: "\
        cortex-m3",
    tests: TESTS,
};

static TESTS: &[TestCase] = &[
    TestCase {
        name: "boot",
        description: "Boot the board in qemu and verify the kernel starts.",
        apps: &[],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &[
                "QEMU MPS2 AN385 (Cortex-M3) initialization complete.",
                "Entering main loop.",
            ],
            timeout: Duration::from_secs(10),
        }],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "c_hello",
        description: "Run the c_hello app and verify it prints \"Hello World!\" over serial.",
        apps: &[App::LibtockC("c_hello")],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["Hello World!"],
            timeout: Duration::from_secs(10),
        }],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
];
