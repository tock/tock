// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

use kernel::utilities::registers::register_bitfields;
use tock_registers::LocalRegisterCopy;

#[cfg(target_arch = "x86")]
use core::arch::asm;

register_bitfields![u32,
    pub EFLAGS[
        /// Carry Flag (CF)
        FLAGS_CF OFFSET(0) NUMBITS(1),
        /// Bit 1 is always 1.
        FLAGS_A1 OFFSET(1) NUMBITS(1),
        /// Parity Flag (PF)
        FLAGS_PF OFFSET(2) NUMBITS(2),
        /// Auxiliary Carry Flag (AF)
        FLAGS_AF OFFSET(4) NUMBITS(2),
        /// Zero Flag (ZF)
        FLAGS_ZF OFFSET(6) NUMBITS(1),
        /// Sign Flag (SF
        FLAGS_SF OFFSET(7) NUMBITS(1),
        /// Trap Flag (TF)
        FLAGS_TF OFFSET(8) NUMBITS(1),
        /// Interrupt Enable Flag (IF)
        FLAGS_IF OFFSET(9) NUMBITS(1),
        /// Direction Flag (DF)
        FLAGS_DF OFFSET(10) NUMBITS(1),
        /// Overflow Flag (OF)
        FLAGS_OF OFFSET(11) NUMBITS(1),
        /// I/O Privilege Level (IOPL) 3
        FLAGS_IOPL OFFSET(12) NUMBITS(2),
        /// Nested Task (NT)
        FLAGS_NT OFFSET(14) NUMBITS(2),
        /// Resume Flag (RF)
        FLAGS_RF OFFSET(16) NUMBITS(1),
        /// Virtual-8086 Mode (VM)
        FLAGS_VM OFFSET(17) NUMBITS(1),
        /// Alignment Check (AC)
        FLAGS_AC OFFSET(18) NUMBITS(1),
        /// Virtual Interrupt Flag (VIF)
        FLAGS_VIF OFFSET(19) NUMBITS(1),
        /// Virtual Interrupt Pending (VIP)
        FLAGS_VIP OFFSET(20) NUMBITS(1),
        /// ID Flag (ID)
        FLAGS_ID OFFSET(21) NUMBITS(11),
    ],
];

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct EFlags(pub LocalRegisterCopy<u32, EFLAGS::Register>);

impl Default for EFlags {
    fn default() -> Self {
        Self::new()
    }
}

impl EFlags {
    /// Create a new SegmentSelector
    /// # Arguments
    ///  * `index` - index in GDT or LDT array.
    ///  * `rpl` - Requested privilege level of the selector
    pub fn new() -> EFlags {
        let mut flags = LocalRegisterCopy::new(0);
        flags.write(EFLAGS::FLAGS_A1::SET);
        EFlags(flags)
    }
}

#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn read() -> EFlags {
    let r: u32;
    unsafe {
        asm!("pushfl; popl {0}", out(reg) r, options(att_syntax));
    }
    EFlags(LocalRegisterCopy::new(r))
}

#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn set(val: EFlags) {
    unsafe {
        asm!("pushl {0}; popfl", in(reg) val.0.get(), options(att_syntax));
    }
}

//For CI only

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn read() -> EFlags {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn set(_val: EFlags) {
    unimplemented!()
}
