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
    pub minstreth: ReadWriteRiscvCsr<usize, minstret::minstreth::Register, MINSTRETH>,
    pub minstret: ReadWriteRiscvCsr<usize, minstret::minstret::Register, MINSTRET>,

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub mcycleh: ReadWriteRiscvCsr<usize, mcycle::mcycleh::Register, MCYCLEH>,
    pub mcycle: ReadWriteRiscvCsr<usize, mcycle::mcycle::Register, MCYCLE>,

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub pmpcfg0: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG0>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub pmpcfg1: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG1>,
    pub pmpcfg2: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG2>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub pmpcfg3: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG3>,
    pub pmpcfg4: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG4>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub pmpcfg5: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG5>,
    pub pmpcfg6: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG6>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub pmpcfg7: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG7>,
    pub pmpcfg8: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG8>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub pmpcfg9: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG9>,
    pub pmpcfg10: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG10>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub pmpcfg11: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG11>,
    pub pmpcfg12: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG12>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub pmpcfg13: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG13>,
    pub pmpcfg14: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG14>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub pmpcfg15: ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register, PMPCFG15>,

    pub pmpaddr0: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR0>,
    pub pmpaddr1: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR1>,
    pub pmpaddr2: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR2>,
    pub pmpaddr3: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR3>,
    pub pmpaddr4: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR4>,
    pub pmpaddr5: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR5>,
    pub pmpaddr6: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR6>,
    pub pmpaddr7: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR7>,
    pub pmpaddr8: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR8>,
    pub pmpaddr9: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR9>,
    pub pmpaddr10: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR10>,
    pub pmpaddr11: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR11>,
    pub pmpaddr12: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR12>,
    pub pmpaddr13: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR13>,
    pub pmpaddr14: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR14>,
    pub pmpaddr15: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR15>,
    pub pmpaddr16: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR16>,
    pub pmpaddr17: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR17>,
    pub pmpaddr18: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR18>,
    pub pmpaddr19: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR19>,
    pub pmpaddr20: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR20>,
    pub pmpaddr21: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR21>,
    pub pmpaddr22: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR22>,
    pub pmpaddr23: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR23>,
    pub pmpaddr24: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR24>,
    pub pmpaddr25: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR25>,
    pub pmpaddr26: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR26>,
    pub pmpaddr27: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR27>,
    pub pmpaddr28: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR28>,
    pub pmpaddr29: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR29>,
    pub pmpaddr30: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR30>,
    pub pmpaddr31: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR31>,
    pub pmpaddr32: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR32>,
    pub pmpaddr33: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR33>,
    pub pmpaddr34: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR34>,
    pub pmpaddr35: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR35>,
    pub pmpaddr36: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR36>,
    pub pmpaddr37: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR37>,
    pub pmpaddr38: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR38>,
    pub pmpaddr39: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR39>,
    pub pmpaddr40: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR40>,
    pub pmpaddr41: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR41>,
    pub pmpaddr42: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR42>,
    pub pmpaddr43: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR43>,
    pub pmpaddr44: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR44>,
    pub pmpaddr45: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR45>,
    pub pmpaddr46: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR46>,
    pub pmpaddr47: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR47>,
    pub pmpaddr48: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR48>,
    pub pmpaddr49: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR49>,
    pub pmpaddr50: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR50>,
    pub pmpaddr51: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR51>,
    pub pmpaddr52: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR52>,
    pub pmpaddr53: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR53>,
    pub pmpaddr54: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR54>,
    pub pmpaddr55: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR55>,
    pub pmpaddr56: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR56>,
    pub pmpaddr57: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR57>,
    pub pmpaddr58: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR58>,
    pub pmpaddr59: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR59>,
    pub pmpaddr60: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR60>,
    pub pmpaddr61: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR61>,
    pub pmpaddr62: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR62>,
    pub pmpaddr63: ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register, PMPADDR63>,

    pub mie: ReadWriteRiscvCsr<usize, mie::mie::Register, MIE>,
    pub mscratch: ReadWriteRiscvCsr<usize, mscratch::mscratch::Register, MSCRATCH>,
    pub mepc: ReadWriteRiscvCsr<usize, mepc::mepc::Register, MEPC>,
    pub mcause: ReadWriteRiscvCsr<usize, mcause::mcause::Register, MCAUSE>,
    pub mtval: ReadWriteRiscvCsr<usize, mtval::mtval::Register, MTVAL>,
    pub mip: ReadWriteRiscvCsr<usize, mip::mip::Register, MIP>,
    pub mtvec: ReadWriteRiscvCsr<usize, mtvec::mtvec::Register, MTVEC>,
    pub mstatus: ReadWriteRiscvCsr<usize, mstatus::mstatus::Register, MSTATUS>,

    pub mseccfg: ReadWriteRiscvCsr<usize, mseccfg::mseccfg::Register, MSECCFG>,
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub mseccfgh: ReadWriteRiscvCsr<usize, mseccfg::mseccfgh::Register, MSECCFGH>,

    pub utvec: ReadWriteRiscvCsr<usize, utvec::utvec::Register, UTVEC>,
    pub stvec: ReadWriteRiscvCsr<usize, stvec::stvec::Register, STVEC>,
}

// Define the "addresses" of each CSR register.
pub const CSR: &CSR = &CSR {
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    minstreth: ReadWriteRiscvCsr::new(),
    minstret: ReadWriteRiscvCsr::new(),

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    mcycleh: ReadWriteRiscvCsr::new(),
    mcycle: ReadWriteRiscvCsr::new(),

    pmpcfg0: ReadWriteRiscvCsr::new(),
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg1: ReadWriteRiscvCsr::new(),
    pmpcfg2: ReadWriteRiscvCsr::new(),
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg3: ReadWriteRiscvCsr::new(),
    pmpcfg4: ReadWriteRiscvCsr::new(),
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg5: ReadWriteRiscvCsr::new(),
    pmpcfg6: ReadWriteRiscvCsr::new(),
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg7: ReadWriteRiscvCsr::new(),
    pmpcfg8: ReadWriteRiscvCsr::new(),
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg9: ReadWriteRiscvCsr::new(),
    pmpcfg10: ReadWriteRiscvCsr::new(),
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg11: ReadWriteRiscvCsr::new(),
    pmpcfg12: ReadWriteRiscvCsr::new(),
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg13: ReadWriteRiscvCsr::new(),
    pmpcfg14: ReadWriteRiscvCsr::new(),
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pmpcfg15: ReadWriteRiscvCsr::new(),

    pmpaddr0: ReadWriteRiscvCsr::new(),
    pmpaddr1: ReadWriteRiscvCsr::new(),
    pmpaddr2: ReadWriteRiscvCsr::new(),
    pmpaddr3: ReadWriteRiscvCsr::new(),
    pmpaddr4: ReadWriteRiscvCsr::new(),
    pmpaddr5: ReadWriteRiscvCsr::new(),
    pmpaddr6: ReadWriteRiscvCsr::new(),
    pmpaddr7: ReadWriteRiscvCsr::new(),
    pmpaddr8: ReadWriteRiscvCsr::new(),
    pmpaddr9: ReadWriteRiscvCsr::new(),
    pmpaddr10: ReadWriteRiscvCsr::new(),
    pmpaddr11: ReadWriteRiscvCsr::new(),
    pmpaddr12: ReadWriteRiscvCsr::new(),
    pmpaddr13: ReadWriteRiscvCsr::new(),
    pmpaddr14: ReadWriteRiscvCsr::new(),
    pmpaddr15: ReadWriteRiscvCsr::new(),
    pmpaddr16: ReadWriteRiscvCsr::new(),
    pmpaddr17: ReadWriteRiscvCsr::new(),
    pmpaddr18: ReadWriteRiscvCsr::new(),
    pmpaddr19: ReadWriteRiscvCsr::new(),
    pmpaddr20: ReadWriteRiscvCsr::new(),
    pmpaddr21: ReadWriteRiscvCsr::new(),
    pmpaddr22: ReadWriteRiscvCsr::new(),
    pmpaddr23: ReadWriteRiscvCsr::new(),
    pmpaddr24: ReadWriteRiscvCsr::new(),
    pmpaddr25: ReadWriteRiscvCsr::new(),
    pmpaddr26: ReadWriteRiscvCsr::new(),
    pmpaddr27: ReadWriteRiscvCsr::new(),
    pmpaddr28: ReadWriteRiscvCsr::new(),
    pmpaddr29: ReadWriteRiscvCsr::new(),
    pmpaddr30: ReadWriteRiscvCsr::new(),
    pmpaddr31: ReadWriteRiscvCsr::new(),
    pmpaddr32: ReadWriteRiscvCsr::new(),
    pmpaddr33: ReadWriteRiscvCsr::new(),
    pmpaddr34: ReadWriteRiscvCsr::new(),
    pmpaddr35: ReadWriteRiscvCsr::new(),
    pmpaddr36: ReadWriteRiscvCsr::new(),
    pmpaddr37: ReadWriteRiscvCsr::new(),
    pmpaddr38: ReadWriteRiscvCsr::new(),
    pmpaddr39: ReadWriteRiscvCsr::new(),
    pmpaddr40: ReadWriteRiscvCsr::new(),
    pmpaddr41: ReadWriteRiscvCsr::new(),
    pmpaddr42: ReadWriteRiscvCsr::new(),
    pmpaddr43: ReadWriteRiscvCsr::new(),
    pmpaddr44: ReadWriteRiscvCsr::new(),
    pmpaddr45: ReadWriteRiscvCsr::new(),
    pmpaddr46: ReadWriteRiscvCsr::new(),
    pmpaddr47: ReadWriteRiscvCsr::new(),
    pmpaddr48: ReadWriteRiscvCsr::new(),
    pmpaddr49: ReadWriteRiscvCsr::new(),
    pmpaddr50: ReadWriteRiscvCsr::new(),
    pmpaddr51: ReadWriteRiscvCsr::new(),
    pmpaddr52: ReadWriteRiscvCsr::new(),
    pmpaddr53: ReadWriteRiscvCsr::new(),
    pmpaddr54: ReadWriteRiscvCsr::new(),
    pmpaddr55: ReadWriteRiscvCsr::new(),
    pmpaddr56: ReadWriteRiscvCsr::new(),
    pmpaddr57: ReadWriteRiscvCsr::new(),
    pmpaddr58: ReadWriteRiscvCsr::new(),
    pmpaddr59: ReadWriteRiscvCsr::new(),
    pmpaddr60: ReadWriteRiscvCsr::new(),
    pmpaddr61: ReadWriteRiscvCsr::new(),
    pmpaddr62: ReadWriteRiscvCsr::new(),
    pmpaddr63: ReadWriteRiscvCsr::new(),

    mie: ReadWriteRiscvCsr::new(),
    mscratch: ReadWriteRiscvCsr::new(),
    mepc: ReadWriteRiscvCsr::new(),
    mcause: ReadWriteRiscvCsr::new(),
    mtval: ReadWriteRiscvCsr::new(),
    mip: ReadWriteRiscvCsr::new(),
    mtvec: ReadWriteRiscvCsr::new(),
    mstatus: ReadWriteRiscvCsr::new(),

    mseccfg: ReadWriteRiscvCsr::new(),
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    mseccfgh: ReadWriteRiscvCsr::new(),

    utvec: ReadWriteRiscvCsr::new(),
    stvec: ReadWriteRiscvCsr::new(),
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

    pub fn pmpconfig_get(&self, index: usize) -> usize {
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

    pub fn pmpconfig_set(&self, index: usize, value: usize) {
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

    pub fn pmpconfig_modify(
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

    pub fn pmpaddr_set(&self, index: usize, value: usize) {
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

    pub fn pmpaddr_get(&self, index: usize) -> usize {
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
}
