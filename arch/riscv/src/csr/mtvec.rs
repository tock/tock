// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::registers::{register_bitfields, LocalRegisterCopy};

// mtvec contains the address(es) of the trap handler
register_bitfields![usize,
    pub mtvec [
        trap_addr OFFSET(2) NUMBITS(crate::XLEN - 2) [],
        mode OFFSET(0) NUMBITS(2) [
            Direct = 0,
            Vectored = 1
        ]
    ]
];

trait MtvecHelpers {
    fn get_trap_address(&self) -> usize;
}

impl MtvecHelpers for LocalRegisterCopy<usize, mtvec::Register> {
    fn get_trap_address(&self) -> usize {
        self.read(mtvec::trap_addr) << 2
    }
}
