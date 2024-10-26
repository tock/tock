// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Shared support for RISC-V architectures.

#![crate_name = "riscv"]
#![crate_type = "rlib"]
#![no_std]

pub mod csr;

#[cfg(all(target_arch = "riscv32", not(doc)))]
pub const XLEN: usize = 32;
#[cfg(all(target_arch = "riscv64", not(doc)))]
pub const XLEN: usize = 64;

// Default to 32 bit if no architecture is specified of if this is being
// compiled for docs or testing on a different architecture.
#[cfg(any(
    doc,
    not(all(
        any(target_arch = "riscv32", target_arch = "riscv64"),
        target_os = "none"
    ))
))]
pub const XLEN: usize = 32;
