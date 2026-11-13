// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! QEMU `virt` machine with a 64-bit RISC-V CPU.

use crate::boards::families::qemu::{Boot, Layout, Qemu, TIMING};
use crate::boards::{BOARDS, Board};

#[linkme::distributed_slice(BOARDS)]
pub static QEMU_RV64_VIRT: Board = Board {
    name: "qemu_rv64_virt",
    dir: "boards/configurations/qemu_rv64_virt/qemu_rv64_virt-test-ci",
    kernel: "target/riscv64imac-unknown-none-elf/release/qemu_rv64_virt-test-ci.bin",
    libtock_c_targets: &["$(QEMU_RV64_VIRT_TOCK_TARGETS)"],
    timing: TIMING,
    open: |tock| {
        Box::new(Qemu::new(
            &QEMU_RV64_VIRT,
            tock,
            Layout {
                board: "qemu_rv64_virt",
                kernel: 0x8000_0000,
                apps: 0x8010_0000,
                boot: Boot::FlashFile,
            },
        ))
    },
};
