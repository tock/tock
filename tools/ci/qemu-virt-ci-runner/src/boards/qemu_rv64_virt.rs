// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use std::time::Duration;

use crate::{TestCase, TestStep};

pub static BOARD: super::Board = super::Board {
    name: "qemu_rv64_virt",
    board_dir: "../../../boards/configurations/qemu_rv64_virt/qemu_rv64_virt-test-ci",
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
        name: "led-odd",
        description: "Run led-odd and verify the screen shows LEDs 1 and 3 on.",
        apps: &["tests/led/led-odd"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["Entering main loop."],
            timeout: Duration::from_secs(30),
        }],
        screenshot_delay: Duration::from_millis(500),
        expected_screen_hash: Some(
            "c4d49a185592f23074ae5b83caa7d506967fbe8f93a547fa60de93bc2ddd282f",
        ),
    },
    TestCase {
        name: "tock-logo",
        description:
            "Run the tock-logo u8g2 demo and verify the screen matches the expected logo.",
        apps: &["tests/u8g2-demos/tock-logo"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["Entering main loop."],
            timeout: Duration::from_secs(30),
        }],
        screenshot_delay: Duration::from_millis(500),
        expected_screen_hash: Some(
            "4150b38c3cf468a388c0a09eb309ac8c480f692599cc0fc3e2684e0fafb5463d",
        ),
    },
    TestCase {
        name: "led-odd-logo",
        description: "Run led-odd and tock-logo together and verify the combined screen output as the LEDs display on the screen.",
        apps: &["tests/led/led-odd", "tests/u8g2-demos/tock-logo"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["Entering main loop."],
            timeout: Duration::from_secs(30),
        }],
        screenshot_delay: Duration::from_millis(500),
        expected_screen_hash: Some(
            "bfec0b5a3c5b72edb6128c7c9b6690b56d7ac93e1047d9ef45a22be0c9b4d4fa",
        ),
    },
    TestCase {
        name: "button_print",
        description: "Use the keyboard to send the character up arrow, and ensure that is translated to button 0 being pressed for userspace.",
        apps: &["tests/button_print"],
        steps: &[
            TestStep::Sleep(Duration::from_millis(500)),
            TestStep::SendKey("up"),
            TestStep::WaitSerialInOrder {
                needles: &["Button Press! Button: 0 Status: 0"],
                timeout: Duration::from_secs(30),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "console",
        description: "Send characters over serial and verify the console app echoes them back.",
        apps: &["tests/console/console"],
        steps: &[
            TestStep::WaitSerialInOrder {
                needles: &["Entering main loop."],
                timeout: Duration::from_secs(30),
            },
            TestStep::Sleep(Duration::from_millis(30)),
            TestStep::SendSerial("a"),
            TestStep::WaitSerialInOrder {
                needles: &["Got character: 'a'"],
                timeout: Duration::from_secs(10),
            },
            TestStep::Sleep(Duration::from_millis(100)),
            TestStep::SendSerial("R"),
            TestStep::WaitSerialInOrder {
                needles: &["Got character: 'R'"],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "hello-pconsole",
        description: "Run c_hello alongside the process console and verify both the app output and the tock$ prompt appear.",
        apps: &["c_hello"],
        steps: &[
            TestStep::Sleep(Duration::from_millis(100)),
            TestStep::SendSerial("\n\r"),
            TestStep::WaitSerialAnyOrder {
                needles: &["Hello World!", "tock$"],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "pconsole-help",
        description: "Send the 'help' command to the process console and verify the help output.",
        apps: &[],
        steps: &[
            TestStep::Sleep(Duration::from_millis(500)),
            TestStep::SendSerial("help\n\r"),
            TestStep::WaitSerialInOrder {
                needles: &[
                    "Valid commands are: help status list stop start fault boot terminate process kernel reset panic console-start console-stop",
                ],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "pconsole-list",
        description: "Run two apps and verify the process console 'list' command shows both, in any scheduler order.",
        apps: &["c_hello", "blink"],
        steps: &[
            TestStep::Sleep(Duration::from_millis(500)),
            TestStep::SendSerial("list\n\r"),
            TestStep::WaitSerialAnyOrder {
                needles: &["blink", "c_hello"],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "exit",
        description: "Run the exit app and confirm it shows as Terminated in the process list.",
        apps: &["tests/exit"],
        steps: &[
            TestStep::WaitSerialInOrder {
                needles: &["Testing exit.", "Exiting."],
                timeout: Duration::from_secs(10),
            },
            TestStep::Sleep(Duration::from_millis(500)),
            TestStep::SendSerial("list\n\r"),
            TestStep::WaitSerialInOrder {
                needles: &["exit", "Terminated"],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "unexpected-rx",
        description: "Send a serial byte before the app starts and verify the kernel handles unexpected RX without crashing.",
        apps: &["c_hello"],
        steps: &[
            TestStep::SendSerial("a"),
            TestStep::WaitSerialInOrder {
                needles: &["Hello World!"],
                timeout: Duration::from_secs(5),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
];
