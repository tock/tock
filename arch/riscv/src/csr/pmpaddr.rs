// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::registers::register_bitfields;

// Default to 32 bit if compiling for debug/testing.
#[cfg(not(target_arch = "riscv64"))]
register_bitfields![usize,
    /// The `pmpaddr` register bitfield.
    ///
    /// This is `XLEN` (32) bits wide on RISCV-32, and `XLEN - 10` (54) bits
    /// wide on RISCV-64: RV64 supports only a 56 bit physical address space,
    /// and addresses here are left-shifted by 2 bits, so the uppermost 10
    /// bits of the 64-bit CSR are reserved (WARL-0). See `PMPADDR_MASK` in
    /// `pmp.rs`.
    pub pmpaddr [
        addr OFFSET(0) NUMBITS(crate::XLEN) []
    ]
];

#[cfg(target_arch = "riscv64")]
register_bitfields![usize,
    /// The `pmpaddr` register bitfield.
    ///
    /// This is `XLEN` (32) bits wide on RISCV-32, and `XLEN - 10` (54) bits
    /// wide on RISCV-64: RV64 supports only a 56 bit physical address space,
    /// and addresses here are left-shifted by 2 bits, so the uppermost 10
    /// bits of the 64-bit CSR are reserved (WARL-0). See `PMPADDR_MASK` in
    /// `pmp.rs`.
    pub pmpaddr [
        addr OFFSET(0) NUMBITS(crate::XLEN - 10) []
    ]
];
