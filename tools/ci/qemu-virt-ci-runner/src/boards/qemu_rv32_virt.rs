// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use std::time::Duration;

use crate::{TestCase, TestStep};

pub static BOARD: super::Board = super::Board {
    name: "qemu_rv32_virt",
    board_dir: "../../../boards/configurations/qemu_rv32_virt/qemu_rv32_virt-test-ci",
    tests: TESTS,
};

static TESTS: &[TestCase] = &[
    TestCase {
        name: "c_hello",
        description: "Run the c_hello app and verify it prints \"Hello World!\" over serial.",
        apps: &["c_hello"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["Hello World!"],
            timeout: Duration::from_secs(30),
        }],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "syscall-return",
        description: "Check syscalls return expected success/error codes.",
        apps: &["tests/syscall-return"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["All tests succeeded"],
            timeout: Duration::from_secs(5),
        }],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "isolated_nonvolatile_storage_read_write",
        description: "Check writing to isolated nonvolatile storage works.",
        apps: &["tests/isolated_nonvolatile_storage/invs_read_write"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["All tests succeeded"],
            timeout: Duration::from_secs(30),
        }],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "sha256",
        description: "Check SHA256 computation works.",
        apps: &["tests/sha"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["SHA computation correct."],
            timeout: Duration::from_secs(5),
        }],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "rng",
        description: "Verify we get multiple rounds of random numbers.",
        apps: &["tests/rng"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["Randomness:", "Randomness:", "Randomness:", "Randomness:"],
            timeout: Duration::from_secs(5),
        }],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
];
