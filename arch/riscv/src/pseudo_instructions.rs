// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Define missing RISC-V pseudo instructions using macros.
//!
//! The RISC-V spec includes numerous pseudo instructions to help with writing
//! RISC-V assembly. However, somewhat bafflingly, there are no pseudo
//! instructions for making operations XLEN bits long. This makes it difficult
//! to write general assembly code that works on both RV32 and RV64 systems,
//! where the assembly only needs to operate on different register sizes.
//!
//! We use the pseudo instructions `lx` and `sx` for loading and storing an
//! entire register, respectively.

/// Loads an XLEN size register.
#[cfg(any(doc, target_arch = "riscv32"))]
#[macro_export]
macro_rules! lx {
    () => {
        "lw "
    };
}

/// Loads an XLEN size register.
#[cfg(target_arch = "riscv64")]
#[macro_export]
macro_rules! lx {
    () => {
        "ld "
    };
}

/// Stores a XLEN size register.
#[cfg(any(doc, target_arch = "riscv32"))]
#[macro_export]
macro_rules! sx {
    () => {
        "sw "
    };
}
/// Stores a XLEN size register.
#[cfg(target_arch = "riscv64")]
#[macro_export]
macro_rules! sx {
    () => {
        "sd "
    };
}
