//! Tock Register interface for using CSR registers.

use riscv_csr::csr::ReadWriteRiscvCsr;

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

#[repr(C)]
pub struct CSR {
    #[cfg(not(feature = "riscv64"))]
    pub minstreth: ReadWriteRiscvCsr<usize, minstret::minstreth::Register>,
    pub minstret: ReadWriteRiscvCsr<usize, minstret::minstret::Register>,
    #[cfg(not(feature = "riscv64"))]
    pub mcycleh: ReadWriteRiscvCsr<usize, mcycle::mcycleh::Register>,
    pub mcycle: ReadWriteRiscvCsr<usize, mcycle::mcycle::Register>,
    #[cfg(not(all(feature = "riscv64", target_os = "none")))]
    pub pmpcfg: [ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register>; 16],
    #[cfg(feature = "riscv64")]
    pub pmpcfg: [ReadWriteRiscvCsr<usize, pmpconfig::pmpcfg::Register>; 8],
    pub pmpaddr: [ReadWriteRiscvCsr<usize, pmpaddr::pmpaddr::Register>; 64],
    pub mie: ReadWriteRiscvCsr<usize, mie::mie::Register>,
    pub mscratch: ReadWriteRiscvCsr<usize, mscratch::mscratch::Register>,
    pub mepc: ReadWriteRiscvCsr<usize, mepc::mepc::Register>,
    pub mcause: ReadWriteRiscvCsr<usize, mcause::mcause::Register>,
    pub mtval: ReadWriteRiscvCsr<usize, mtval::mtval::Register>,
    pub mip: ReadWriteRiscvCsr<usize, mip::mip::Register>,
    pub mtvec: ReadWriteRiscvCsr<usize, mtvec::mtvec::Register>,
    pub stvec: ReadWriteRiscvCsr<usize, stvec::stvec::Register>,
    pub utvec: ReadWriteRiscvCsr<usize, utvec::utvec::Register>,
    pub mstatus: ReadWriteRiscvCsr<usize, mstatus::mstatus::Register>,
}

// Define the "addresses" of each CSR register.
pub const CSR: &CSR = &CSR {
    #[cfg(not(feature = "riscv64"))]
    minstreth: ReadWriteRiscvCsr::new(riscv_csr::csr::MINSTRETH),
    minstret: ReadWriteRiscvCsr::new(riscv_csr::csr::MINSTRET),
    #[cfg(not(feature = "riscv64"))]
    mcycleh: ReadWriteRiscvCsr::new(riscv_csr::csr::MCYCLEH),
    mcycle: ReadWriteRiscvCsr::new(riscv_csr::csr::MCYCLE),
    mie: ReadWriteRiscvCsr::new(riscv_csr::csr::MIE),
    mtvec: ReadWriteRiscvCsr::new(riscv_csr::csr::MTVEC),
    mstatus: ReadWriteRiscvCsr::new(riscv_csr::csr::MSTATUS),
    utvec: ReadWriteRiscvCsr::new(riscv_csr::csr::UTVEC),
    stvec: ReadWriteRiscvCsr::new(riscv_csr::csr::STVEC),
    mscratch: ReadWriteRiscvCsr::new(riscv_csr::csr::MSCRATCH),
    mepc: ReadWriteRiscvCsr::new(riscv_csr::csr::MEPC),
    mcause: ReadWriteRiscvCsr::new(riscv_csr::csr::MCAUSE),
    mtval: ReadWriteRiscvCsr::new(riscv_csr::csr::MTVAL),
    mip: ReadWriteRiscvCsr::new(riscv_csr::csr::MIP),

    pmpcfg: [
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG0),
        #[cfg(not(feature = "riscv64"))]
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG1),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG2),
        #[cfg(not(feature = "riscv64"))]
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG3),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG4),
        #[cfg(not(feature = "riscv64"))]
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG5),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG6),
        #[cfg(not(feature = "riscv64"))]
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG7),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG8),
        #[cfg(not(feature = "riscv64"))]
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG9),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG10),
        #[cfg(not(feature = "riscv64"))]
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG11),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG12),
        #[cfg(not(feature = "riscv64"))]
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG13),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG14),
        #[cfg(not(feature = "riscv64"))]
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG15),
    ],
    pmpaddr: [
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR0),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR1),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR2),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR3),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR4),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR5),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR6),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR7),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR8),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR9),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR10),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR11),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR12),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR13),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR14),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR15),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR16),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR17),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR18),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR19),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR20),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR21),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR22),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR23),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR24),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR25),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR26),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR27),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR28),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR29),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR30),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR31),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR32),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR33),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR34),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR35),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR36),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR37),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR38),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR39),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR40),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR41),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR42),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR43),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR44),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR45),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR46),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR47),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR48),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR49),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR50),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR51),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR52),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR53),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR54),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR55),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR56),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR57),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR58),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR59),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR60),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR61),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR62),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPADDR63),
    ],
};

impl CSR {
    // resets the cycle counter to 0
    #[cfg(not(feature = "riscv64"))]
    pub fn reset_cycle_counter(&self) {
        // Write lower first so that we don't overflow before writing the upper
        CSR.mcycle.write(mcycle::mcycle::mcycle.val(0));
        CSR.mcycleh.write(mcycle::mcycleh::mcycleh.val(0));
    }

    // resets the cycle counter to 0
    #[cfg(feature = "riscv64")]
    pub fn reset_cycle_counter(&self) {
        CSR.mcycle.write(mcycle::mcycle::mcycle.val(0));
    }

    // reads the cycle counter
    #[cfg(not(feature = "riscv64"))]
    pub fn read_cycle_counter(&self) -> u64 {
        let (mut top, mut bot): (usize, usize);

        // This should only ever loop twice
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
    #[cfg(feature = "riscv64")]
    pub fn read_cycle_counter(&self) -> u64 {
        CSR.mcycle.read(mcycle::mcycle::mcycle)
    }
}
