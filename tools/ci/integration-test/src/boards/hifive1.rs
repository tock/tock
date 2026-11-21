// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! HiFive1 Rev B, emulated by QEMU's `sifive_e` machine.

use crate::boards::families::qemu::{Boot, Layout, Qemu, TIMING};
use crate::boards::{BOARDS, Board};

#[linkme::distributed_slice(BOARDS)]
pub static HIFIVE1: Board = Board {
    name: "hifive1",
    dir: "boards/hifive1",
    kernel: "target/riscv32imac-unknown-none-elf/release/hifive1.elf",
    libtock_c_targets: &["rv32imac|rv32imac.0x20040080.0x80002800|0x20040080|0x80002800"],
    timing: TIMING,
    open: |tock| {
        Box::new(Qemu::new(
            &HIFIVE1,
            tock,
            Layout {
                board: "hifive1b",
                kernel: 0x2001_0000,
                apps: 0x2004_0000,
                boot: Boot::KernelAndApps { flash: 0x2000_0000 },
            },
        ))
    },
};
