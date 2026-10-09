// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use std::time::Duration;

use crate::{TestCase, TestStep};

pub static BOARD: super::Board = super::Board {
    name: "earlgrey_cw310",
    board_dir: "../../../boards/opentitan/earlgrey-cw310",
    tock_targets: "\
        rv32imc|rv32imc.0x20030080.0x10005000|0x20030080|0x10005000",
    tests: TESTS,
};

// n.b. No app-based test case here (yet): loading an app at `APP_ADDRESS`
// builds and installs cleanly, but the process never runs under QEMU's
// `opentitan` machine, even via the raw `-device loader` invocation this
// board used before switching to tockloader. QEMU prints "Failed to set
// flash memory protection: NOSUPPORT" during boot, which looks like a
// pre-existing gap in QEMU's `opentitan` flash_ctrl model, not something
// introduced by this board's CI setup.
static TESTS: &[TestCase] = &[TestCase {
    name: "boot",
    description: "Boot the board in qemu and verify the kernel starts.",
    apps: &[],
    steps: &[TestStep::WaitSerialInOrder {
        needles: &["OpenTitan initialisation complete.", "Entering main loop"],
        timeout: Duration::from_secs(10),
    }],
    screenshot_delay: Duration::from_millis(0),
    expected_screen_hash: None,
}];
