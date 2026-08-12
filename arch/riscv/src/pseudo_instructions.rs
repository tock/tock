// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Define missing RISC-V pseudo instructions using assembly macros.
//!
//! The RISC-V spec includes numerous pseudo instructions to help with writing
//! RISC-V assembly. However, somewhat bafflingly, there are no pseudo
//! instructions for making operations XLEN bits long. This makes it difficult
//! to write general assembly code that works on both RV32 and RV64 systems,
//! where the assembly only needs to operate on different XLENs.
//!
//! `xlen_macros!` defines macros `lx` and `sx`, which function as
//! pseudoinstructions for loading and storing XLEN-sized values.

/// Generate `asm!()` macros to create register-sized load and store
/// instructions.
///
/// On RISCV-32 this creates two psuedoinstructions:
/// 1. `sx` -> `sw`
/// 2. `lx` -> `lw`
///
/// On RISCV-64 this creates two psuedoinstructions:
/// 1. `sx` -> `sd`
/// 2. `lx` -> `ld`
///
/// These apply globally, and so only need to be included once in the project.
/// For example:
///
/// ```rust,ignore
/// use core::arch::naked_asm;
/// use riscv::xlen_macros;
/// naked_asm!(
///     xlen_macros!(),
///     "
///     ...asm code here...
///     "
/// );
/// ```
#[cfg(target_arch = "riscv32")]
#[macro_export]
macro_rules! xlen_macros[() => [r"
    .macro sx src, dest
        sw \src, \dest
    .endm
    .macro lx dest, src
        lw \dest, \src
    .endm
"]];

/// Generate `asm!()` macros to create register-sized load and store
/// instructions.
///
/// On RISCV-32 this creates two psuedoinstructions:
/// 1. `sx` -> `sw`
/// 2. `lx` -> `lw`
///
/// On RISCV-64 this creates two psuedoinstructions:
/// 1. `sx` -> `sd`
/// 2. `lx` -> `ld`
///
/// These apply globally, and so only need to be included once in the project.
/// For example:
///
/// ```rust,ignore
/// use core::arch::naked_asm;
/// use riscv::xlen_macros;
/// naked_asm!(
///     xlen_macros!(),
///     "
///     ...asm code here...
///     "
/// );
/// ```
#[cfg(target_arch = "riscv64")]
#[macro_export]
macro_rules! xlen_macros[() => [r"
    .macro sx src, dest
        sd \src, \dest
    .endm
    .macro lx dest, src
        ld \dest, \src
    .endm
"]];
