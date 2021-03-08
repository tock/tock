//! Tock Register interface for using CSR registers.

use core::marker::PhantomData;
use tock_registers::registers::{
    Field, FieldValue, IntLike, LocalRegisterCopy, RegisterLongName, TryFromValue,
};

use riscv_csr::csr::{
    ReadWriteRiscvPmpCsr, MCAUSE, MCYCLE, MCYCLEH, MEPC, MIE, MINSTRET, MINSTRETH, MIP, MSCRATCH,
    MSTATUS, MTVAL, MTVEC, STVEC, UTVEC,
};
use riscv_csr::riscv_csr;

pub mod mcause;
pub mod mcycle;
pub mod mepc;
pub mod mie;
pub mod minstret;
pub mod mip;
pub mod mscratch;
pub mod mstatus;
pub mod mtval;
pub mod mtvec;
pub mod pmpaddr;
pub mod pmpconfig;
pub mod stvec;
pub mod utvec;

riscv_csr!(MINSTRETH, ReadWriteRiscvCsrMinstreth);
riscv_csr!(MINSTRET, ReadWriteRiscvCsrMinstret);
riscv_csr!(MCYCLEH, ReadWriteRiscvCsrMcycleh);
riscv_csr!(MCYCLE, ReadWriteRiscvCsrMcycle);
riscv_csr!(MIE, ReadWriteRiscvCsrMie);
riscv_csr!(MTVEC, ReadWriteRiscvCsrMtvec);
riscv_csr!(MSCRATCH, ReadWriteRiscvCsrMscratch);
riscv_csr!(MEPC, ReadWriteRiscvCsrMepc);
riscv_csr!(MCAUSE, ReadWriteRiscvCsrMcause);
riscv_csr!(MTVAL, ReadWriteRiscvCsrMtval);
riscv_csr!(MIP, ReadWriteRiscvCsrMip);
riscv_csr!(MSTATUS, ReadWriteRiscvCsrMstatus);
riscv_csr!(STVEC, ReadWriteRiscvCsrStvec);
riscv_csr!(UTVEC, ReadWriteRiscvCsrUtvec);

// NOTE! We default to 32 bit if this is being compiled for debug/testing. We do
// this by using `cfg` that check for either the architecture is `riscv32` (true
// if we are compiling for a rv32i target), OR if the target OS is set to
// something (as it would be if compiled for a host OS).

#[repr(C)]
pub struct CSR {
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub minstreth: ReadWriteRiscvCsrMinstreth<usize, minstret::minstreth::Register>,
    pub minstret: ReadWriteRiscvCsrMinstret<usize, minstret::minstret::Register>,

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub mcycleh: ReadWriteRiscvCsrMcycleh<usize, mcycle::mcycleh::Register>,
    pub mcycle: ReadWriteRiscvCsrMcycle<usize, mcycle::mcycle::Register>,

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub pmpcfg: [ReadWriteRiscvPmpCsr<usize, pmpconfig::pmpcfg::Register>; 16],
    #[cfg(target_arch = "riscv64")]
    pub pmpcfg: [ReadWriteRiscvPmpCsr<usize, pmpconfig::pmpcfg::Register>; 8],

    pub pmpaddr: [ReadWriteRiscvPmpCsr<usize, pmpaddr::pmpaddr::Register>; 64],

    pub mie: ReadWriteRiscvCsrMie<usize, mie::mie::Register>,
    pub mtvec: ReadWriteRiscvCsrMtvec<usize, mtvec::mtvec::Register>,
    pub mscratch: ReadWriteRiscvCsrMscratch<usize, mscratch::mscratch::Register>,
    pub mepc: ReadWriteRiscvCsrMepc<usize, mepc::mepc::Register>,
    pub mcause: ReadWriteRiscvCsrMcause<usize, mcause::mcause::Register>,
    pub mtval: ReadWriteRiscvCsrMtval<usize, mtval::mtval::Register>,
    pub mip: ReadWriteRiscvCsrMip<usize, mip::mip::Register>,
    pub mstatus: ReadWriteRiscvCsrMstatus<usize, mstatus::mstatus::Register>,

    pub stvec: ReadWriteRiscvCsrStvec<usize, stvec::stvec::Register>,
    pub utvec: ReadWriteRiscvCsrUtvec<usize, utvec::utvec::Register>,
}

// Define the "addresses" of each CSR register.
pub const CSR: &CSR = &CSR {
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    minstreth: ReadWriteRiscvCsrMinstreth::new(),
    minstret: ReadWriteRiscvCsrMinstret::new(),

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    mcycleh: ReadWriteRiscvCsrMcycleh::new(),
    mcycle: ReadWriteRiscvCsrMcycle::new(),

    pmpcfg: [
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG0),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG1),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG2),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG3),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG4),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG5),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG6),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG7),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG8),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG9),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG10),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG11),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG12),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG13),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG14),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPCFG15),
    ],

    pmpaddr: [
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR0),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR1),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR2),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR3),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR4),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR5),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR6),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR7),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR8),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR9),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR10),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR11),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR12),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR13),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR14),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR15),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR16),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR17),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR18),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR19),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR20),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR21),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR22),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR23),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR24),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR25),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR26),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR27),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR28),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR29),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR30),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR31),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR32),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR33),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR34),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR35),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR36),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR37),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR38),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR39),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR40),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR41),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR42),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR43),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR44),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR45),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR46),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR47),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR48),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR49),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR50),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR51),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR52),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR53),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR54),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR55),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR56),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR57),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR58),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR59),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR60),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR61),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR62),
        ReadWriteRiscvPmpCsr::new(riscv_csr::csr::PMPADDR63),
    ],

    mie: ReadWriteRiscvCsrMie::new(),
    mtvec: ReadWriteRiscvCsrMtvec::new(),
    mscratch: ReadWriteRiscvCsrMscratch::new(),
    mepc: ReadWriteRiscvCsrMepc::new(),
    mcause: ReadWriteRiscvCsrMcause::new(),
    mtval: ReadWriteRiscvCsrMtval::new(),
    mip: ReadWriteRiscvCsrMip::new(),
    mstatus: ReadWriteRiscvCsrMstatus::new(),

    stvec: ReadWriteRiscvCsrStvec::new(),
    utvec: ReadWriteRiscvCsrUtvec::new(),
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
}
