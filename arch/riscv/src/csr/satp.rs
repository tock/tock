// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Google LLC 2024.

use kernel::utilities::registers::{register_bitfields, LocalRegisterCopy};

// satp contains the root PTE

#[cfg(target_arch = "riscv32")]
register_bitfields![usize,
    pub satp [
        ppn     OFFSET(0)   NUMBITS(22) [],
        asid    OFFSET(22)  NUMBITS(9)  [],
        mode    OFFSET(31)  NUMBITS(1)  [
            BARE = 0,
            Sv32 = 1,
        ]
    ]
];

#[cfg(any(target_arch = "riscv64", not(target_os = "none")))]
register_bitfields![usize,
    pub satp [
        ppn     OFFSET(0)   NUMBITS(44) [],
        asid    OFFSET(44)  NUMBITS(16) [],
        mode    OFFSET(60)  NUMBITS(4)  [
            BARE = 0,
            Sv39 = 8,
            Sv48 = 9,
            Sv57 = 10,
            Sv64 = 11,
        ],
    ]
];

pub trait PPNAddr {
    fn get_ppn_as_addr(&self) -> usize;
}

impl PPNAddr for LocalRegisterCopy<usize, satp::Register> {
    fn get_ppn_as_addr(&self) -> usize {
        self.read(satp::ppn) << 12
    }
}
