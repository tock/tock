// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::registers::register_bitfields;

// Default to 32 bit if compiling for debug/testing.
#[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
register_bitfields![usize,
    pub pmpaddr [
        addr OFFSET(0) NUMBITS(crate::XLEN) []
    ]
];

#[cfg(target_arch = "riscv64")]
register_bitfields![usize,
    pub pmpaddr [
        addr OFFSET(0) NUMBITS(crate::XLEN - 10) []
    ]
];
