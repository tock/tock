// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock Register interface for using CSR registers.

use riscv_csr::csr::{
    ReadWriteRiscvCsr, MCAUSE, MCYCLE, MCYCLEH, MEPC, MIE, MINSTRET, MINSTRETH, MIP, MSCRATCH,
    MSECCFG, MSECCFGH, MSTATUS, MTVAL, MTVEC, PMPADDR0, PMPADDR1, PMPADDR10, PMPADDR11, PMPADDR12,
    PMPADDR13, PMPADDR14, PMPADDR15, PMPADDR16, PMPADDR17, PMPADDR18, PMPADDR19, PMPADDR2,
    PMPADDR20, PMPADDR21, PMPADDR22, PMPADDR23, PMPADDR24, PMPADDR25, PMPADDR26, PMPADDR27,
    PMPADDR28, PMPADDR29, PMPADDR3, PMPADDR30, PMPADDR31, PMPADDR32, PMPADDR33, PMPADDR34,
    PMPADDR35, PMPADDR36, PMPADDR37, PMPADDR38, PMPADDR39, PMPADDR4, PMPADDR40, PMPADDR41,
    PMPADDR42, PMPADDR43, PMPADDR44, PMPADDR45, PMPADDR46, PMPADDR47, PMPADDR48, PMPADDR49,
    PMPADDR5, PMPADDR50, PMPADDR51, PMPADDR52, PMPADDR53, PMPADDR54, PMPADDR55, PMPADDR56,
    PMPADDR57, PMPADDR58, PMPADDR59, PMPADDR6, PMPADDR60, PMPADDR61, PMPADDR62, PMPADDR63,
    PMPADDR7, PMPADDR8, PMPADDR9, PMPCFG0, PMPCFG1, PMPCFG10, PMPCFG11, PMPCFG12, PMPCFG13,
    PMPCFG14, PMPCFG15, PMPCFG2, PMPCFG3, PMPCFG4, PMPCFG5, PMPCFG6, PMPCFG7, PMPCFG8, PMPCFG9,
    STVEC, UTVEC,
};
use tock_registers::fields::FieldValue;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

pub mod mcause;
pub mod mcycle;
pub mod mepc;
pub mod mie;
pub mod minstret;
pub mod mip;
pub mod mscratch;
pub mod mseccfg;
pub mod mstatus;
pub mod mtval;
pub mod mtvec;
pub mod pmpaddr;
pub mod pmpconfig;
pub mod stvec;
pub mod utvec;

// NOTE! We default to 32 bit if this is being compiled for debug/testing. We do
// this by using `cfg` that check for either the architecture is `riscv32` (true
// if we are compiling for a rv32i target), OR if the target OS is set to
// something (as it would be if compiled for a host OS).

pub struct CSR {
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    minstreth: ReadWriteRiscvCsr<usize, minstret::minstreth::Register, MINSTRETH>,
    minstret: ReadWriteRiscvCsr<usize, minstret::minstret::Register, MINSTRET>,

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    mcycleh: ReadWriteRiscvCsr<usize, mcycle::mcycleh::Register, MCYCLEH>,
    mcycle: ReadWriteRiscvCsr<usize, mcycle::mcycle::Register, MCYCLE>,

    pmpcfg0: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG0>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg1: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG1>,
    pmpcfg2: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG2>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg3: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG3>,
    pmpcfg4: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG4>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg5: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG5>,
    pmpcfg6: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG6>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg7: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG7>,
    pmpcfg8: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG8>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg9: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG9>,
    pmpcfg10: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG10>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg11: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG11>,
    pmpcfg12: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG12>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg13: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG13>,
    pmpcfg14: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG14>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg15: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG15>,

    pmpaddr0: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR0>,
    pmpaddr1: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR1>,
    pmpaddr2: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR2>,
    pmpaddr3: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR3>,
    pmpaddr4: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR4>,
    pmpaddr5: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR5>,
    pmpaddr6: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR6>,
    pmpaddr7: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR7>,
    pmpaddr8: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR8>,
    pmpaddr9: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR9>,
    pmpaddr10: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR10>,
    pmpaddr11: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR11>,
    pmpaddr12: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR12>,
    pmpaddr13: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR13>,
    pmpaddr14: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR14>,
    pmpaddr15: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR15>,
    pmpaddr16: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR16>,
    pmpaddr17: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR17>,
    pmpaddr18: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR18>,
    pmpaddr19: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR19>,
    pmpaddr20: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR20>,
    pmpaddr21: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR21>,
    pmpaddr22: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR22>,
    pmpaddr23: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR23>,
    pmpaddr24: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR24>,
    pmpaddr25: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR25>,
    pmpaddr26: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR26>,
    pmpaddr27: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR27>,
    pmpaddr28: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR28>,
    pmpaddr29: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR29>,
    pmpaddr30: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR30>,
    pmpaddr31: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR31>,
    pmpaddr32: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR32>,
    pmpaddr33: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR33>,
    pmpaddr34: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR34>,
    pmpaddr35: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR35>,
    pmpaddr36: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR36>,
    pmpaddr37: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR37>,
    pmpaddr38: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR38>,
    pmpaddr39: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR39>,
    pmpaddr40: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR40>,
    pmpaddr41: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR41>,
    pmpaddr42: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR42>,
    pmpaddr43: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR43>,
    pmpaddr44: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR44>,
    pmpaddr45: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR45>,
    pmpaddr46: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR46>,
    pmpaddr47: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR47>,
    pmpaddr48: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR48>,
    pmpaddr49: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR49>,
    pmpaddr50: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR50>,
    pmpaddr51: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR51>,
    pmpaddr52: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR52>,
    pmpaddr53: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR53>,
    pmpaddr54: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR54>,
    pmpaddr55: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR55>,
    pmpaddr56: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR56>,
    pmpaddr57: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR57>,
    pmpaddr58: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR58>,
    pmpaddr59: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR59>,
    pmpaddr60: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR60>,
    pmpaddr61: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR61>,
    pmpaddr62: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR62>,
    pmpaddr63: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR63>,

    mie: ReadWriteRiscvCsr<usize, mie::mie::Register, MIE>,
    mscratch: ReadWriteRiscvCsr<usize, mscratch::mscratch::Register, MSCRATCH>,
    mepc: ReadWriteRiscvCsr<usize, mepc::mepc::Register, MEPC>,
    mcause: ReadWriteRiscvCsr<usize, mcause::mcause::Register, MCAUSE>,
    mtval: ReadWriteRiscvCsr<usize, mtval::mtval::Register, MTVAL>,
    mip: ReadWriteRiscvCsr<usize, mip::mip::Register, MIP>,
    mtvec: ReadWriteRiscvCsr<usize, mtvec::mtvec::Register, MTVEC>,
    mstatus: ReadWriteRiscvCsr<usize, mstatus::mstatus::Register, MSTATUS>,

    mseccfg: ReadWriteRiscvCsr<usize, mseccfg::mseccfg::Register, MSECCFG>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    mseccfgh: ReadWriteRiscvCsr<usize, mseccfg::mseccfgh::Register, MSECCFGH>,

    utvec: ReadWriteRiscvCsr<usize, utvec::utvec::Register, UTVEC>,
    stvec: ReadWriteRiscvCsr<usize, stvec::stvec::Register, STVEC>,
}

// Define the "addresses" of each CSR register.
pub const CSR: &CSR = &CSR {
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    minstreth: unsafe { ReadWriteRiscvCsr::new() },
    minstret: unsafe { ReadWriteRiscvCsr::new() },

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    mcycleh: unsafe { ReadWriteRiscvCsr::new() },
    mcycle: unsafe { ReadWriteRiscvCsr::new() },

    pmpcfg0: unsafe { ReadWriteRiscvCsr::new() },
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg1: unsafe { ReadWriteRiscvCsr::new() },
    pmpcfg2: unsafe { ReadWriteRiscvCsr::new() },
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg3: unsafe { ReadWriteRiscvCsr::new() },
    pmpcfg4: unsafe { ReadWriteRiscvCsr::new() },
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg5: unsafe { ReadWriteRiscvCsr::new() },
    pmpcfg6: unsafe { ReadWriteRiscvCsr::new() },
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg7: unsafe { ReadWriteRiscvCsr::new() },
    pmpcfg8: unsafe { ReadWriteRiscvCsr::new() },
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg9: unsafe { ReadWriteRiscvCsr::new() },
    pmpcfg10: unsafe { ReadWriteRiscvCsr::new() },
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg11: unsafe { ReadWriteRiscvCsr::new() },
    pmpcfg12: unsafe { ReadWriteRiscvCsr::new() },
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg13: unsafe { ReadWriteRiscvCsr::new() },
    pmpcfg14: unsafe { ReadWriteRiscvCsr::new() },
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg15: unsafe { ReadWriteRiscvCsr::new() },

    pmpaddr0: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr1: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr2: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr3: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr4: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr5: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr6: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr7: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr8: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr9: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr10: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr11: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr12: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr13: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr14: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr15: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr16: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr17: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr18: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr19: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr20: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr21: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr22: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr23: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr24: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr25: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr26: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr27: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr28: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr29: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr30: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr31: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr32: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr33: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr34: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr35: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr36: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr37: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr38: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr39: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr40: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr41: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr42: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr43: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr44: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr45: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr46: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr47: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr48: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr49: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr50: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr51: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr52: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr53: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr54: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr55: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr56: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr57: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr58: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr59: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr60: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr61: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr62: unsafe { ReadWriteRiscvCsr::new() },
    pmpaddr63: unsafe { ReadWriteRiscvCsr::new() },

    mie: unsafe { ReadWriteRiscvCsr::new() },
    mscratch: unsafe { ReadWriteRiscvCsr::new() },
    mepc: unsafe { ReadWriteRiscvCsr::new() },
    mcause: unsafe { ReadWriteRiscvCsr::new() },
    mtval: unsafe { ReadWriteRiscvCsr::new() },
    mip: unsafe { ReadWriteRiscvCsr::new() },
    mtvec: unsafe { ReadWriteRiscvCsr::new() },
    mstatus: unsafe { ReadWriteRiscvCsr::new() },

    mseccfg: unsafe { ReadWriteRiscvCsr::new() },
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    mseccfgh: unsafe { ReadWriteRiscvCsr::new() },

    utvec: unsafe { ReadWriteRiscvCsr::new() },
    stvec: unsafe { ReadWriteRiscvCsr::new() },
};

impl CSR {
    // resets the cycle counter to 0
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub fn reset_cycle_counter(&self) {
        // Write lower first so that we don't overflow before writing the upper
        CSR.mcycle.write(mcycle::mcycle::mcycle.val(0));
        CSR.mcycleh.write(mcycle::mcycleh::mcycleh.val(0));
    }

    // resets the cycle counter to 0
    #[cfg(target_arch = "riscv64")]
    pub fn reset_cycle_counter(&self) {
        CSR.mcycle.write(mcycle::mcycle::mcycle.val(0));
    }

    // reads the cycle counter
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub fn read_cycle_counter(&self) -> u64 {
        let (mut top, mut bot): (usize, usize);

        // Need to handle the potential for rollover between reading the lower
        // and upper bits. We do this by reading twice, and seeing if the upper
        // bits change between reads. This should only ever loop at most twice.
        loop {
            top = CSR.mcycleh.read(mcycle::mcycleh::mcycleh);
            bot = CSR.mcycle.read(mcycle::mcycle::mcycle);
            if top == CSR.mcycleh.read(mcycle::mcycleh::mcycleh) {
                break;
            }
        }

        (top as u64).checked_shl(32).unwrap() + bot as u64
    }

    // reads the cycle counter
    #[cfg(target_arch = "riscv64")]
    pub fn read_cycle_counter(&self) -> u64 {
        CSR.mcycle.read(mcycle::mcycle::mcycle)
    }

    // unsafe as access to non-existant pmpcfgX register can cause a trap
    pub unsafe fn pmpconfig_get(&self, index: usize) -> usize {
        match index {
            0 => self.pmpcfg0.get(),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            1 => self.pmpcfg1.get(),
            2 => self.pmpcfg2.get(),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            3 => self.pmpcfg3.get(),
            4 => self.pmpcfg4.get(),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            5 => self.pmpcfg5.get(),
            6 => self.pmpcfg6.get(),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            7 => self.pmpcfg7.get(),
            8 => self.pmpcfg8.get(),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            9 => self.pmpcfg9.get(),
            10 => self.pmpcfg10.get(),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            11 => self.pmpcfg11.get(),
            12 => self.pmpcfg12.get(),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            13 => self.pmpcfg13.get(),
            14 => self.pmpcfg14.get(),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            15 => self.pmpcfg15.get(),
            _ => unreachable!(),
        }
    }

    pub unsafe fn pmpconfig_set(&self, index: usize, value: usize) {
        match index {
            0 => self.pmpcfg0.set(value),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            1 => self.pmpcfg1.set(value),
            2 => self.pmpcfg2.set(value),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            3 => self.pmpcfg3.set(value),
            4 => self.pmpcfg4.set(value),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            5 => self.pmpcfg5.set(value),
            6 => self.pmpcfg6.set(value),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            7 => self.pmpcfg7.set(value),
            8 => self.pmpcfg8.set(value),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            9 => self.pmpcfg9.set(value),
            10 => self.pmpcfg10.set(value),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            11 => self.pmpcfg11.set(value),
            12 => self.pmpcfg12.set(value),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            13 => self.pmpcfg13.set(value),
            14 => self.pmpcfg14.set(value),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            15 => self.pmpcfg15.set(value),
            _ => unreachable!(),
        }
    }

    pub unsafe fn pmpconfig_modify(
        &self,
        index: usize,
        field: FieldValue<usize, pmpconfig::pmpcfg::Register>,
    ) {
        match index {
            0 => self.pmpcfg0.modify(field),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            1 => self.pmpcfg1.modify(field),
            2 => self.pmpcfg2.modify(field),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            3 => self.pmpcfg3.modify(field),
            4 => self.pmpcfg4.modify(field),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            5 => self.pmpcfg5.modify(field),
            6 => self.pmpcfg6.modify(field),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            7 => self.pmpcfg7.modify(field),
            8 => self.pmpcfg8.modify(field),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            9 => self.pmpcfg9.modify(field),
            10 => self.pmpcfg10.modify(field),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            11 => self.pmpcfg11.modify(field),
            12 => self.pmpcfg12.modify(field),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            13 => self.pmpcfg13.modify(field),
            14 => self.pmpcfg14.modify(field),
            #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
            15 => self.pmpcfg15.modify(field),
            _ => unreachable!(),
        }
    }

    pub unsafe fn pmpaddr_set(&self, index: usize, value: usize) {
        match index {
            0 => self.pmpaddr0.set(value),
            1 => self.pmpaddr1.set(value),
            2 => self.pmpaddr2.set(value),
            3 => self.pmpaddr3.set(value),
            4 => self.pmpaddr4.set(value),
            5 => self.pmpaddr5.set(value),
            6 => self.pmpaddr6.set(value),
            7 => self.pmpaddr7.set(value),
            8 => self.pmpaddr8.set(value),
            9 => self.pmpaddr9.set(value),
            10 => self.pmpaddr10.set(value),
            11 => self.pmpaddr11.set(value),
            12 => self.pmpaddr12.set(value),
            13 => self.pmpaddr13.set(value),
            14 => self.pmpaddr14.set(value),
            15 => self.pmpaddr15.set(value),
            16 => self.pmpaddr16.set(value),
            17 => self.pmpaddr17.set(value),
            18 => self.pmpaddr18.set(value),
            19 => self.pmpaddr19.set(value),
            20 => self.pmpaddr20.set(value),
            21 => self.pmpaddr21.set(value),
            22 => self.pmpaddr22.set(value),
            23 => self.pmpaddr23.set(value),
            24 => self.pmpaddr24.set(value),
            25 => self.pmpaddr25.set(value),
            26 => self.pmpaddr26.set(value),
            27 => self.pmpaddr27.set(value),
            28 => self.pmpaddr28.set(value),
            29 => self.pmpaddr29.set(value),
            30 => self.pmpaddr30.set(value),
            31 => self.pmpaddr31.set(value),
            32 => self.pmpaddr32.set(value),
            33 => self.pmpaddr33.set(value),
            34 => self.pmpaddr34.set(value),
            35 => self.pmpaddr35.set(value),
            36 => self.pmpaddr36.set(value),
            37 => self.pmpaddr37.set(value),
            38 => self.pmpaddr38.set(value),
            39 => self.pmpaddr39.set(value),
            40 => self.pmpaddr40.set(value),
            41 => self.pmpaddr41.set(value),
            42 => self.pmpaddr42.set(value),
            43 => self.pmpaddr43.set(value),
            44 => self.pmpaddr44.set(value),
            45 => self.pmpaddr45.set(value),
            46 => self.pmpaddr46.set(value),
            47 => self.pmpaddr47.set(value),
            48 => self.pmpaddr48.set(value),
            49 => self.pmpaddr49.set(value),
            50 => self.pmpaddr50.set(value),
            51 => self.pmpaddr51.set(value),
            52 => self.pmpaddr52.set(value),
            53 => self.pmpaddr53.set(value),
            54 => self.pmpaddr54.set(value),
            55 => self.pmpaddr55.set(value),
            56 => self.pmpaddr56.set(value),
            57 => self.pmpaddr57.set(value),
            58 => self.pmpaddr58.set(value),
            59 => self.pmpaddr59.set(value),
            60 => self.pmpaddr60.set(value),
            61 => self.pmpaddr61.set(value),
            62 => self.pmpaddr62.set(value),
            63 => self.pmpaddr63.set(value),
            _ => unreachable!(),
        }
    }

    // unsafe as access to non-existant pmpaddrX register can cause a trap
    pub unsafe fn pmpaddr_get(&self, index: usize) -> usize {
        match index {
            0 => self.pmpaddr0.get(),
            1 => self.pmpaddr1.get(),
            2 => self.pmpaddr2.get(),
            3 => self.pmpaddr3.get(),
            4 => self.pmpaddr4.get(),
            5 => self.pmpaddr5.get(),
            6 => self.pmpaddr6.get(),
            7 => self.pmpaddr7.get(),
            8 => self.pmpaddr8.get(),
            9 => self.pmpaddr9.get(),
            10 => self.pmpaddr10.get(),
            11 => self.pmpaddr11.get(),
            12 => self.pmpaddr12.get(),
            13 => self.pmpaddr13.get(),
            14 => self.pmpaddr14.get(),
            15 => self.pmpaddr15.get(),
            16 => self.pmpaddr16.get(),
            17 => self.pmpaddr17.get(),
            18 => self.pmpaddr18.get(),
            19 => self.pmpaddr19.get(),
            20 => self.pmpaddr20.get(),
            21 => self.pmpaddr21.get(),
            22 => self.pmpaddr22.get(),
            23 => self.pmpaddr23.get(),
            24 => self.pmpaddr24.get(),
            25 => self.pmpaddr25.get(),
            26 => self.pmpaddr26.get(),
            27 => self.pmpaddr27.get(),
            28 => self.pmpaddr28.get(),
            29 => self.pmpaddr29.get(),
            30 => self.pmpaddr30.get(),
            31 => self.pmpaddr31.get(),
            32 => self.pmpaddr32.get(),
            33 => self.pmpaddr33.get(),
            34 => self.pmpaddr34.get(),
            35 => self.pmpaddr35.get(),
            36 => self.pmpaddr36.get(),
            37 => self.pmpaddr37.get(),
            38 => self.pmpaddr38.get(),
            39 => self.pmpaddr39.get(),
            40 => self.pmpaddr40.get(),
            41 => self.pmpaddr41.get(),
            42 => self.pmpaddr42.get(),
            43 => self.pmpaddr43.get(),
            44 => self.pmpaddr44.get(),
            45 => self.pmpaddr45.get(),
            46 => self.pmpaddr46.get(),
            47 => self.pmpaddr47.get(),
            48 => self.pmpaddr48.get(),
            49 => self.pmpaddr49.get(),
            50 => self.pmpaddr50.get(),
            51 => self.pmpaddr51.get(),
            52 => self.pmpaddr52.get(),
            53 => self.pmpaddr53.get(),
            54 => self.pmpaddr54.get(),
            55 => self.pmpaddr55.get(),
            56 => self.pmpaddr56.get(),
            57 => self.pmpaddr57.get(),
            58 => self.pmpaddr58.get(),
            59 => self.pmpaddr59.get(),
            60 => self.pmpaddr60.get(),
            61 => self.pmpaddr61.get(),
            62 => self.pmpaddr62.get(),
            63 => self.pmpaddr63.get(),
            _ => unreachable!(),
        }
    }

    // CSR accessor methods

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub unsafe fn minstreth(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, minstret::minstreth::Register, MINSTRETH> {
        &self.minstreth
    }

    pub unsafe fn minstret(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, minstret::minstret::Register, MINSTRET> {
        &self.minstret
    }

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub unsafe fn mcycleh(&self) -> &ReadWriteRiscvCsr<usize, mcycle::mcycleh::Register, MCYCLEH> {
        &self.mcycleh
    }

    pub unsafe fn mcycle(&self) -> &ReadWriteRiscvCsr<usize, mcycle::mcycle::Register, MCYCLE> {
        &self.mcycle
    }

    pub unsafe fn pmpcfg0(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG0> {
        &self.pmpcfg0
    }

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub unsafe fn pmpcfg1(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG1> {
        &self.pmpcfg1
    }

    pub unsafe fn pmpcfg2(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG2> {
        &self.pmpcfg2
    }

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub unsafe fn pmpcfg3(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG3> {
        &self.pmpcfg3
    }

    pub unsafe fn pmpcfg4(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG4> {
        &self.pmpcfg4
    }

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub unsafe fn pmpcfg5(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG5> {
        &self.pmpcfg5
    }

    pub unsafe fn pmpcfg6(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG6> {
        &self.pmpcfg6
    }

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub unsafe fn pmpcfg7(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG7> {
        &self.pmpcfg7
    }

    pub unsafe fn pmpcfg8(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG8> {
        &self.pmpcfg8
    }

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub unsafe fn pmpcfg9(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG9> {
        &self.pmpcfg9
    }

    pub unsafe fn pmpcfg10(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG10> {
        &self.pmpcfg10
    }

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub unsafe fn pmpcfg11(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG11> {
        &self.pmpcfg11
    }

    pub unsafe fn pmpcfg12(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG12> {
        &self.pmpcfg12
    }

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub unsafe fn pmpcfg13(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG13> {
        &self.pmpcfg13
    }

    pub unsafe fn pmpcfg14(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG14> {
        &self.pmpcfg14
    }

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub unsafe fn pmpcfg15(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG15> {
        &self.pmpcfg15
    }

    pub unsafe fn pmpaddr0(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR0> {
        &self.pmpaddr0
    }

    pub unsafe fn pmpaddr1(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR1> {
        &self.pmpaddr1
    }

    pub unsafe fn pmpaddr2(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR2> {
        &self.pmpaddr2
    }

    pub unsafe fn pmpaddr3(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR3> {
        &self.pmpaddr3
    }

    pub unsafe fn pmpaddr4(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR4> {
        &self.pmpaddr4
    }

    pub unsafe fn pmpaddr5(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR5> {
        &self.pmpaddr5
    }

    pub unsafe fn pmpaddr6(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR6> {
        &self.pmpaddr6
    }

    pub unsafe fn pmpaddr7(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR7> {
        &self.pmpaddr7
    }

    pub unsafe fn pmpaddr8(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR8> {
        &self.pmpaddr8
    }

    pub unsafe fn pmpaddr9(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR9> {
        &self.pmpaddr9
    }

    pub unsafe fn pmpaddr10(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR10> {
        &self.pmpaddr10
    }

    pub unsafe fn pmpaddr11(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR11> {
        &self.pmpaddr11
    }

    pub unsafe fn pmpaddr12(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR12> {
        &self.pmpaddr12
    }

    pub unsafe fn pmpaddr13(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR13> {
        &self.pmpaddr13
    }

    pub unsafe fn pmpaddr14(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR14> {
        &self.pmpaddr14
    }

    pub unsafe fn pmpaddr15(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR15> {
        &self.pmpaddr15
    }

    pub unsafe fn pmpaddr16(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR16> {
        &self.pmpaddr16
    }

    pub unsafe fn pmpaddr17(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR17> {
        &self.pmpaddr17
    }

    pub unsafe fn pmpaddr18(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR18> {
        &self.pmpaddr18
    }

    pub unsafe fn pmpaddr19(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR19> {
        &self.pmpaddr19
    }

    pub unsafe fn pmpaddr20(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR20> {
        &self.pmpaddr20
    }

    pub unsafe fn pmpaddr21(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR21> {
        &self.pmpaddr21
    }

    pub unsafe fn pmpaddr22(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR22> {
        &self.pmpaddr22
    }

    pub unsafe fn pmpaddr23(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR23> {
        &self.pmpaddr23
    }

    pub unsafe fn pmpaddr24(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR24> {
        &self.pmpaddr24
    }

    pub unsafe fn pmpaddr25(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR25> {
        &self.pmpaddr25
    }

    pub unsafe fn pmpaddr26(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR26> {
        &self.pmpaddr26
    }

    pub unsafe fn pmpaddr27(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR27> {
        &self.pmpaddr27
    }

    pub unsafe fn pmpaddr28(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR28> {
        &self.pmpaddr28
    }

    pub unsafe fn pmpaddr29(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR29> {
        &self.pmpaddr29
    }

    pub unsafe fn pmpaddr30(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR30> {
        &self.pmpaddr30
    }

    pub unsafe fn pmpaddr31(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR31> {
        &self.pmpaddr31
    }

    pub unsafe fn pmpaddr32(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR32> {
        &self.pmpaddr32
    }

    pub unsafe fn pmpaddr33(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR33> {
        &self.pmpaddr33
    }

    pub unsafe fn pmpaddr34(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR34> {
        &self.pmpaddr34
    }

    pub unsafe fn pmpaddr35(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR35> {
        &self.pmpaddr35
    }

    pub unsafe fn pmpaddr36(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR36> {
        &self.pmpaddr36
    }

    pub unsafe fn pmpaddr37(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR37> {
        &self.pmpaddr37
    }

    pub unsafe fn pmpaddr38(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR38> {
        &self.pmpaddr38
    }

    pub unsafe fn pmpaddr39(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR39> {
        &self.pmpaddr39
    }

    pub unsafe fn pmpaddr40(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR40> {
        &self.pmpaddr40
    }

    pub unsafe fn pmpaddr41(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR41> {
        &self.pmpaddr41
    }

    pub unsafe fn pmpaddr42(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR42> {
        &self.pmpaddr42
    }

    pub unsafe fn pmpaddr43(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR43> {
        &self.pmpaddr43
    }

    pub unsafe fn pmpaddr44(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR44> {
        &self.pmpaddr44
    }

    pub unsafe fn pmpaddr45(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR45> {
        &self.pmpaddr45
    }

    pub unsafe fn pmpaddr46(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR46> {
        &self.pmpaddr46
    }

    pub unsafe fn pmpaddr47(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR47> {
        &self.pmpaddr47
    }

    pub unsafe fn pmpaddr48(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR48> {
        &self.pmpaddr48
    }

    pub unsafe fn pmpaddr49(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR49> {
        &self.pmpaddr49
    }

    pub unsafe fn pmpaddr50(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR50> {
        &self.pmpaddr50
    }

    pub unsafe fn pmpaddr51(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR51> {
        &self.pmpaddr51
    }

    pub unsafe fn pmpaddr52(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR52> {
        &self.pmpaddr52
    }

    pub unsafe fn pmpaddr53(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR53> {
        &self.pmpaddr53
    }

    pub unsafe fn pmpaddr54(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR54> {
        &self.pmpaddr54
    }

    pub unsafe fn pmpaddr55(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR55> {
        &self.pmpaddr55
    }

    pub unsafe fn pmpaddr56(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR56> {
        &self.pmpaddr56
    }

    pub unsafe fn pmpaddr57(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR57> {
        &self.pmpaddr57
    }

    pub unsafe fn pmpaddr58(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR58> {
        &self.pmpaddr58
    }

    pub unsafe fn pmpaddr59(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR59> {
        &self.pmpaddr59
    }

    pub unsafe fn pmpaddr60(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR60> {
        &self.pmpaddr60
    }

    pub unsafe fn pmpaddr61(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR61> {
        &self.pmpaddr61
    }

    pub unsafe fn pmpaddr62(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR62> {
        &self.pmpaddr62
    }

    pub unsafe fn pmpaddr63(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR63> {
        &self.pmpaddr63
    }

    pub unsafe fn mie(&self) -> &ReadWriteRiscvCsr<usize, mie::mie::Register, MIE> {
        &self.mie
    }

    pub unsafe fn mscratch(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, mscratch::mscratch::Register, MSCRATCH> {
        &self.mscratch
    }

    pub unsafe fn mepc(&self) -> &ReadWriteRiscvCsr<usize, mepc::mepc::Register, MEPC> {
        &self.mepc
    }

    pub unsafe fn mcause(&self) -> &ReadWriteRiscvCsr<usize, mcause::mcause::Register, MCAUSE> {
        &self.mcause
    }

    pub unsafe fn mtval(&self) -> &ReadWriteRiscvCsr<usize, mtval::mtval::Register, MTVAL> {
        &self.mtval
    }

    pub unsafe fn mip(&self) -> &ReadWriteRiscvCsr<usize, mip::mip::Register, MIP> {
        &self.mip
    }

    pub unsafe fn mtvec(&self) -> &ReadWriteRiscvCsr<usize, mtvec::mtvec::Register, MTVEC> {
        &self.mtvec
    }

    pub unsafe fn mstatus(&self) -> &ReadWriteRiscvCsr<usize, mstatus::mstatus::Register, MSTATUS> {
        &self.mstatus
    }

    pub unsafe fn mseccfg(&self) -> &ReadWriteRiscvCsr<usize, mseccfg::mseccfg::Register, MSECCFG> {
        &self.mseccfg
    }

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub unsafe fn mseccfgh(
        &self,
    ) -> &ReadWriteRiscvCsr<usize, mseccfg::mseccfgh::Register, MSECCFGH> {
        &self.mseccfgh
    }

    pub unsafe fn utvec(&self) -> &ReadWriteRiscvCsr<usize, utvec::utvec::Register, UTVEC> {
        &self.utvec
    }

    pub unsafe fn stvec(&self) -> &ReadWriteRiscvCsr<usize, stvec::stvec::Register, STVEC> {
        &self.stvec
    }
}
