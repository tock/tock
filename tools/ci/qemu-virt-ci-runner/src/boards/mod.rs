// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

pub mod qemu_rv64_virt;

pub struct Board {
    /// Short identifier used with --board on the command line.
    pub name: &'static str,
    /// Path to the board directory (where `make run` is executed), relative to
    /// the qemu-virt-ci-runner working directory.
    pub board_dir: &'static str,
    /// All test cases defined for this board.
    pub tests: &'static [crate::TestCase],
}

pub static BOARDS: &[&Board] = &[&qemu_rv64_virt::BOARD];
