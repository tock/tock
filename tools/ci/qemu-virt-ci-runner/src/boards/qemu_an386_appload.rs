// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use std::time::Duration;

use crate::{App, SerialPort, TestCase, TestStep};

pub static BOARD: super::Board = super::Board {
    name: "an386_appload",
    board_dir: "../../../boards/configurations/qemu_arm_mps2_an386/qemu_arm_mps2_an386-test-apploader",
    tock_targets: "\
        cortex-m4",
    tests: TESTS,
};

static TESTS: &[TestCase] = &[
    TestCase {
        name: "hello_loop",
        description: "Make sure the hello_loop app works.",
        apps: &[App::LibtockC("tests/hello_loop")],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["Hello", "Hello", "Hello", "Hello", "Hello"],
            timeout: Duration::from_secs(10),
        }],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
        needs_serial1: false,
    },
    TestCase {
        name: "appload-ymodem-hello-loop",
        description: "",
        apps: &[
            App::LibtockC("c_hello"),
            App::LibtockC("tests/dynamic-app-loading/loader-ymodem"),
        ],
        steps: &[
            TestStep::WaitSerialAnyOrder {
                needles: &["Hello World!", "[AppLoader] ymodem app started"],
                timeout: Duration::from_secs(10),
            },
            TestStep::SendFileYmodem {
                port: SerialPort::Secondary,
                path: "../../../../libtock-c/examples/tests/hello_loop/build/cortex-m4/cortex-m4.tbf",
            },
            TestStep::WaitSerialInOrder {
                needles: &["Hello", "Hello", "Hello", "Hello", "Hello"],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
        needs_serial1: true,
    },
    TestCase {
        name: "appload-ymodem-twoapps",
        description: "",
        apps: &[App::LibtockC("tests/dynamic-app-loading/loader-ymodem")],
        steps: &[
            TestStep::WaitSerialAnyOrder {
                needles: &["[AppLoader] ymodem app started"],
                timeout: Duration::from_secs(10),
            },
            TestStep::SendFileYmodem {
                port: SerialPort::Secondary,
                path: "../../../../libtock-c/examples/c_hello/build/cortex-m4/cortex-m4.tbf",
            },
            TestStep::WaitSerialInOrder {
                needles: &["Hello World!"],
                timeout: Duration::from_secs(10),
            },
            TestStep::SendFileYmodem {
                port: SerialPort::Secondary,
                path: "../../../../libtock-c/examples/tests/printf_long/build/cortex-m4/cortex-m4.tbf",
            },
            TestStep::WaitSerialInOrder {
                needles: &[
                    "This test makes sure that a greater than 64 byte message can be printed.",
                    "And a short message.",
                ],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
        needs_serial1: true,
    },
];
