// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Shared support for RISC-V architectures.

#![crate_name = "riscv"]
#![crate_type = "rlib"]
#![no_std]

pub mod csr;

// Default to 32 bit if no architecture is specified of if this is being
// compiled for docs or testing on a different architecture.
pub const XLEN: usize = if cfg!(target_arch = "riscv64") {
    64
} else {
    32
};
