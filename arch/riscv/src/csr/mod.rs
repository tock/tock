//! Tock Register interface for using CSR registers.

use core::marker::PhantomData;
use tock_registers::registers::{IntLike, RegisterLongName};

use riscv_csr::csr::{
    MCAUSE, MCYCLE, MCYCLEH, MEPC, MIE, MINSTRET, MINSTRETH, MIP, MSCRATCH, MSTATUS, MTVAL, MTVEC,
    PMPADDR0, PMPADDR1, PMPADDR10, PMPADDR11, PMPADDR12, PMPADDR13, PMPADDR14, PMPADDR15,
    PMPADDR16, PMPADDR17, PMPADDR18, PMPADDR19, PMPADDR2, PMPADDR20, PMPADDR21, PMPADDR22,
    PMPADDR23, PMPADDR24, PMPADDR25, PMPADDR26, PMPADDR27, PMPADDR28, PMPADDR29, PMPADDR3,
    PMPADDR30, PMPADDR31, PMPADDR32, PMPADDR33, PMPADDR34, PMPADDR35, PMPADDR36, PMPADDR37,
    PMPADDR38, PMPADDR39, PMPADDR4, PMPADDR40, PMPADDR41, PMPADDR42, PMPADDR43, PMPADDR44,
    PMPADDR45, PMPADDR46, PMPADDR47, PMPADDR48, PMPADDR49, PMPADDR5, PMPADDR50, PMPADDR51,
    PMPADDR52, PMPADDR53, PMPADDR54, PMPADDR55, PMPADDR56, PMPADDR57, PMPADDR58, PMPADDR59,
    PMPADDR6, PMPADDR60, PMPADDR61, PMPADDR62, PMPADDR63, PMPADDR7, PMPADDR8, PMPADDR9, PMPCFG0,
    PMPCFG1, PMPCFG10, PMPCFG11, PMPCFG12, PMPCFG13, PMPCFG14, PMPCFG15, PMPCFG2, PMPCFG3, PMPCFG4,
    PMPCFG5, PMPCFG6, PMPCFG7, PMPCFG8, PMPCFG9, STVEC, UTVEC,
};
use riscv_csr::riscv_csr;

pub use riscv_csr::csr::RISCVCSRReadWrite;

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

riscv_csr!(PMPCFG0, ReadWriteRiscvCsrPmpcfg0);
#[cfg(not(target_arch = "riscv64"))]
riscv_csr!(PMPCFG1, ReadWriteRiscvCsrPmpcfg1);
riscv_csr!(PMPCFG2, ReadWriteRiscvCsrPmpcfg2);
#[cfg(not(target_arch = "riscv64"))]
riscv_csr!(PMPCFG3, ReadWriteRiscvCsrPmpcfg3);
riscv_csr!(PMPCFG4, ReadWriteRiscvCsrPmpcfg4);
#[cfg(not(target_arch = "riscv64"))]
riscv_csr!(PMPCFG5, ReadWriteRiscvCsrPmpcfg5);
riscv_csr!(PMPCFG6, ReadWriteRiscvCsrPmpcfg6);
#[cfg(not(target_arch = "riscv64"))]
riscv_csr!(PMPCFG7, ReadWriteRiscvCsrPmpcfg7);
riscv_csr!(PMPCFG8, ReadWriteRiscvCsrPmpcfg8);
#[cfg(not(target_arch = "riscv64"))]
riscv_csr!(PMPCFG9, ReadWriteRiscvCsrPmpcfg9);
riscv_csr!(PMPCFG10, ReadWriteRiscvCsrPmpcfg10);
#[cfg(not(target_arch = "riscv64"))]
riscv_csr!(PMPCFG11, ReadWriteRiscvCsrPmpcfg11);
riscv_csr!(PMPCFG12, ReadWriteRiscvCsrPmpcfg12);
#[cfg(not(target_arch = "riscv64"))]
riscv_csr!(PMPCFG13, ReadWriteRiscvCsrPmpcfg13);
riscv_csr!(PMPCFG14, ReadWriteRiscvCsrPmpcfg14);
#[cfg(not(target_arch = "riscv64"))]
riscv_csr!(PMPCFG15, ReadWriteRiscvCsrPmpcfg15);

riscv_csr!(PMPADDR0, ReadWriteRiscvCsrPmpaddr0);
riscv_csr!(PMPADDR1, ReadWriteRiscvCsrPmpaddr1);
riscv_csr!(PMPADDR2, ReadWriteRiscvCsrPmpaddr2);
riscv_csr!(PMPADDR3, ReadWriteRiscvCsrPmpaddr3);
riscv_csr!(PMPADDR4, ReadWriteRiscvCsrPmpaddr4);
riscv_csr!(PMPADDR5, ReadWriteRiscvCsrPmpaddr5);
riscv_csr!(PMPADDR6, ReadWriteRiscvCsrPmpaddr6);
riscv_csr!(PMPADDR7, ReadWriteRiscvCsrPmpaddr7);
riscv_csr!(PMPADDR8, ReadWriteRiscvCsrPmpaddr8);
riscv_csr!(PMPADDR9, ReadWriteRiscvCsrPmpaddr9);
riscv_csr!(PMPADDR10, ReadWriteRiscvCsrPmpaddr10);
riscv_csr!(PMPADDR11, ReadWriteRiscvCsrPmpaddr11);
riscv_csr!(PMPADDR12, ReadWriteRiscvCsrPmpaddr12);
riscv_csr!(PMPADDR13, ReadWriteRiscvCsrPmpaddr13);
riscv_csr!(PMPADDR14, ReadWriteRiscvCsrPmpaddr14);
riscv_csr!(PMPADDR15, ReadWriteRiscvCsrPmpaddr15);
riscv_csr!(PMPADDR16, ReadWriteRiscvCsrPmpaddr16);
riscv_csr!(PMPADDR17, ReadWriteRiscvCsrPmpaddr17);
riscv_csr!(PMPADDR18, ReadWriteRiscvCsrPmpaddr18);
riscv_csr!(PMPADDR19, ReadWriteRiscvCsrPmpaddr19);
riscv_csr!(PMPADDR20, ReadWriteRiscvCsrPmpaddr20);
riscv_csr!(PMPADDR21, ReadWriteRiscvCsrPmpaddr21);
riscv_csr!(PMPADDR22, ReadWriteRiscvCsrPmpaddr22);
riscv_csr!(PMPADDR23, ReadWriteRiscvCsrPmpaddr23);
riscv_csr!(PMPADDR24, ReadWriteRiscvCsrPmpaddr24);
riscv_csr!(PMPADDR25, ReadWriteRiscvCsrPmpaddr25);
riscv_csr!(PMPADDR26, ReadWriteRiscvCsrPmpaddr26);
riscv_csr!(PMPADDR27, ReadWriteRiscvCsrPmpaddr27);
riscv_csr!(PMPADDR28, ReadWriteRiscvCsrPmpaddr28);
riscv_csr!(PMPADDR29, ReadWriteRiscvCsrPmpaddr29);
riscv_csr!(PMPADDR30, ReadWriteRiscvCsrPmpaddr30);
riscv_csr!(PMPADDR31, ReadWriteRiscvCsrPmpaddr31);
riscv_csr!(PMPADDR32, ReadWriteRiscvCsrPmpaddr32);
riscv_csr!(PMPADDR33, ReadWriteRiscvCsrPmpaddr33);
riscv_csr!(PMPADDR34, ReadWriteRiscvCsrPmpaddr34);
riscv_csr!(PMPADDR35, ReadWriteRiscvCsrPmpaddr35);
riscv_csr!(PMPADDR36, ReadWriteRiscvCsrPmpaddr36);
riscv_csr!(PMPADDR37, ReadWriteRiscvCsrPmpaddr37);
riscv_csr!(PMPADDR38, ReadWriteRiscvCsrPmpaddr38);
riscv_csr!(PMPADDR39, ReadWriteRiscvCsrPmpaddr39);
riscv_csr!(PMPADDR40, ReadWriteRiscvCsrPmpaddr40);
riscv_csr!(PMPADDR41, ReadWriteRiscvCsrPmpaddr41);
riscv_csr!(PMPADDR42, ReadWriteRiscvCsrPmpaddr42);
riscv_csr!(PMPADDR43, ReadWriteRiscvCsrPmpaddr43);
riscv_csr!(PMPADDR44, ReadWriteRiscvCsrPmpaddr44);
riscv_csr!(PMPADDR45, ReadWriteRiscvCsrPmpaddr45);
riscv_csr!(PMPADDR46, ReadWriteRiscvCsrPmpaddr46);
riscv_csr!(PMPADDR47, ReadWriteRiscvCsrPmpaddr47);
riscv_csr!(PMPADDR48, ReadWriteRiscvCsrPmpaddr48);
riscv_csr!(PMPADDR49, ReadWriteRiscvCsrPmpaddr49);
riscv_csr!(PMPADDR50, ReadWriteRiscvCsrPmpaddr50);
riscv_csr!(PMPADDR51, ReadWriteRiscvCsrPmpaddr51);
riscv_csr!(PMPADDR52, ReadWriteRiscvCsrPmpaddr52);
riscv_csr!(PMPADDR53, ReadWriteRiscvCsrPmpaddr53);
riscv_csr!(PMPADDR54, ReadWriteRiscvCsrPmpaddr54);
riscv_csr!(PMPADDR55, ReadWriteRiscvCsrPmpaddr55);
riscv_csr!(PMPADDR56, ReadWriteRiscvCsrPmpaddr56);
riscv_csr!(PMPADDR57, ReadWriteRiscvCsrPmpaddr57);
riscv_csr!(PMPADDR58, ReadWriteRiscvCsrPmpaddr58);
riscv_csr!(PMPADDR59, ReadWriteRiscvCsrPmpaddr59);
riscv_csr!(PMPADDR60, ReadWriteRiscvCsrPmpaddr60);
riscv_csr!(PMPADDR61, ReadWriteRiscvCsrPmpaddr61);
riscv_csr!(PMPADDR62, ReadWriteRiscvCsrPmpaddr62);
riscv_csr!(PMPADDR63, ReadWriteRiscvCsrPmpaddr63);

// NOTE! We default to 32 bit if this is being compiled for debug/testing. We do
// this by using `cfg` that check for either the architecture is `riscv32` (true
// if we are compiling for a rv32i target), OR if the target OS is set to
// something (as it would be if compiled for a host OS).

#[repr(C)]
pub struct CSR<'a> {
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub minstreth: &'a dyn RISCVCSRReadWrite<usize, minstret::minstreth::Register>,
    pub minstret: &'a dyn RISCVCSRReadWrite<usize, minstret::minstret::Register>,

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub mcycleh: &'a dyn RISCVCSRReadWrite<usize, mcycle::mcycleh::Register>,
    pub mcycle: &'a dyn RISCVCSRReadWrite<usize, mcycle::mcycle::Register>,

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    pub pmpcfg: [&'a dyn RISCVCSRReadWrite<usize, pmpconfig::pmpcfg::Register>; 16],
    #[cfg(target_arch = "riscv64")]
    pub pmpcfg: [&'a dyn RISCVCSRReadWrite<usize, pmpconfig::pmpcfg::Register>; 8],

    pub pmpaddr: [&'a dyn RISCVCSRReadWrite<usize, pmpaddr::pmpaddr::Register>; 64],

    pub mie: &'a dyn RISCVCSRReadWrite<usize, mie::mie::Register>,
    pub mtvec: &'a dyn RISCVCSRReadWrite<usize, mtvec::mtvec::Register>,
    pub mscratch: &'a dyn RISCVCSRReadWrite<usize, mscratch::mscratch::Register>,
    pub mepc: &'a dyn RISCVCSRReadWrite<usize, mepc::mepc::Register>,
    pub mcause: &'a dyn RISCVCSRReadWrite<usize, mcause::mcause::Register>,
    pub mtval: &'a dyn RISCVCSRReadWrite<usize, mtval::mtval::Register>,
    pub mip: &'a dyn RISCVCSRReadWrite<usize, mip::mip::Register>,
    pub mstatus: &'a dyn RISCVCSRReadWrite<usize, mstatus::mstatus::Register>,

    pub stvec: &'a dyn RISCVCSRReadWrite<usize, stvec::stvec::Register>,
    pub utvec: &'a dyn RISCVCSRReadWrite<usize, utvec::utvec::Register>,
}

// Define the "addresses" of each CSR register.
pub const CSR: &CSR = &CSR {
    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    minstreth: &ReadWriteRiscvCsrMinstreth::new(),
    minstret: &ReadWriteRiscvCsrMinstret::new(),

    #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
    mcycleh: &ReadWriteRiscvCsrMcycleh::new(),
    mcycle: &ReadWriteRiscvCsrMcycle::new(),

    pmpcfg: [
        &ReadWriteRiscvCsrPmpcfg0::new(),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        &ReadWriteRiscvCsrPmpcfg1::new(),
        &ReadWriteRiscvCsrPmpcfg2::new(),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        &ReadWriteRiscvCsrPmpcfg3::new(),
        &ReadWriteRiscvCsrPmpcfg4::new(),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        &ReadWriteRiscvCsrPmpcfg5::new(),
        &ReadWriteRiscvCsrPmpcfg6::new(),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        &ReadWriteRiscvCsrPmpcfg7::new(),
        &ReadWriteRiscvCsrPmpcfg8::new(),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        &ReadWriteRiscvCsrPmpcfg9::new(),
        &ReadWriteRiscvCsrPmpcfg10::new(),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        &ReadWriteRiscvCsrPmpcfg11::new(),
        &ReadWriteRiscvCsrPmpcfg12::new(),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        &ReadWriteRiscvCsrPmpcfg13::new(),
        &ReadWriteRiscvCsrPmpcfg14::new(),
        #[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
        &ReadWriteRiscvCsrPmpcfg15::new(),
    ],

    pmpaddr: [
        &ReadWriteRiscvCsrPmpaddr0::new(),
        &ReadWriteRiscvCsrPmpaddr1::new(),
        &ReadWriteRiscvCsrPmpaddr2::new(),
        &ReadWriteRiscvCsrPmpaddr3::new(),
        &ReadWriteRiscvCsrPmpaddr4::new(),
        &ReadWriteRiscvCsrPmpaddr5::new(),
        &ReadWriteRiscvCsrPmpaddr6::new(),
        &ReadWriteRiscvCsrPmpaddr7::new(),
        &ReadWriteRiscvCsrPmpaddr8::new(),
        &ReadWriteRiscvCsrPmpaddr9::new(),
        &ReadWriteRiscvCsrPmpaddr10::new(),
        &ReadWriteRiscvCsrPmpaddr11::new(),
        &ReadWriteRiscvCsrPmpaddr12::new(),
        &ReadWriteRiscvCsrPmpaddr13::new(),
        &ReadWriteRiscvCsrPmpaddr14::new(),
        &ReadWriteRiscvCsrPmpaddr15::new(),
        &ReadWriteRiscvCsrPmpaddr16::new(),
        &ReadWriteRiscvCsrPmpaddr17::new(),
        &ReadWriteRiscvCsrPmpaddr18::new(),
        &ReadWriteRiscvCsrPmpaddr19::new(),
        &ReadWriteRiscvCsrPmpaddr20::new(),
        &ReadWriteRiscvCsrPmpaddr21::new(),
        &ReadWriteRiscvCsrPmpaddr22::new(),
        &ReadWriteRiscvCsrPmpaddr23::new(),
        &ReadWriteRiscvCsrPmpaddr24::new(),
        &ReadWriteRiscvCsrPmpaddr25::new(),
        &ReadWriteRiscvCsrPmpaddr26::new(),
        &ReadWriteRiscvCsrPmpaddr27::new(),
        &ReadWriteRiscvCsrPmpaddr28::new(),
        &ReadWriteRiscvCsrPmpaddr29::new(),
        &ReadWriteRiscvCsrPmpaddr30::new(),
        &ReadWriteRiscvCsrPmpaddr31::new(),
        &ReadWriteRiscvCsrPmpaddr32::new(),
        &ReadWriteRiscvCsrPmpaddr33::new(),
        &ReadWriteRiscvCsrPmpaddr34::new(),
        &ReadWriteRiscvCsrPmpaddr35::new(),
        &ReadWriteRiscvCsrPmpaddr36::new(),
        &ReadWriteRiscvCsrPmpaddr37::new(),
        &ReadWriteRiscvCsrPmpaddr38::new(),
        &ReadWriteRiscvCsrPmpaddr39::new(),
        &ReadWriteRiscvCsrPmpaddr40::new(),
        &ReadWriteRiscvCsrPmpaddr41::new(),
        &ReadWriteRiscvCsrPmpaddr42::new(),
        &ReadWriteRiscvCsrPmpaddr43::new(),
        &ReadWriteRiscvCsrPmpaddr44::new(),
        &ReadWriteRiscvCsrPmpaddr45::new(),
        &ReadWriteRiscvCsrPmpaddr46::new(),
        &ReadWriteRiscvCsrPmpaddr47::new(),
        &ReadWriteRiscvCsrPmpaddr48::new(),
        &ReadWriteRiscvCsrPmpaddr49::new(),
        &ReadWriteRiscvCsrPmpaddr50::new(),
        &ReadWriteRiscvCsrPmpaddr51::new(),
        &ReadWriteRiscvCsrPmpaddr52::new(),
        &ReadWriteRiscvCsrPmpaddr53::new(),
        &ReadWriteRiscvCsrPmpaddr54::new(),
        &ReadWriteRiscvCsrPmpaddr55::new(),
        &ReadWriteRiscvCsrPmpaddr56::new(),
        &ReadWriteRiscvCsrPmpaddr57::new(),
        &ReadWriteRiscvCsrPmpaddr58::new(),
        &ReadWriteRiscvCsrPmpaddr59::new(),
        &ReadWriteRiscvCsrPmpaddr60::new(),
        &ReadWriteRiscvCsrPmpaddr61::new(),
        &ReadWriteRiscvCsrPmpaddr62::new(),
        &ReadWriteRiscvCsrPmpaddr63::new(),
    ],

    mie: &ReadWriteRiscvCsrMie::new(),
    mtvec: &ReadWriteRiscvCsrMtvec::new(),
    mscratch: &ReadWriteRiscvCsrMscratch::new(),
    mepc: &ReadWriteRiscvCsrMepc::new(),
    mcause: &ReadWriteRiscvCsrMcause::new(),
    mtval: &ReadWriteRiscvCsrMtval::new(),
    mip: &ReadWriteRiscvCsrMip::new(),
    mstatus: &ReadWriteRiscvCsrMstatus::new(),

    stvec: &ReadWriteRiscvCsrStvec::new(),
    utvec: &ReadWriteRiscvCsrUtvec::new(),
};

impl CSR<'_> {
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
