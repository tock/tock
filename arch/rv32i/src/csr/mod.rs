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
    pub minstreth: ReadWriteRiscvCsr<u32, minstret::minstreth::Register>,
    pub minstret: ReadWriteRiscvCsr<u32, minstret::minstret::Register>,
    pub mcycleh: ReadWriteRiscvCsr<u32, mcycle::mcycleh::Register>,
    pub mcycle: ReadWriteRiscvCsr<u32, mcycle::mcycle::Register>,
    pub pmpcfg: [ReadWriteRiscvCsr<u32, pmpconfig::pmpcfg::Register>; 4],
    pub pmpaddr: [ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>; 16],
    pub mie: ReadWriteRiscvCsr<u32, mie::mie::Register>,
    pub mscratch: ReadWriteRiscvCsr<u32, mscratch::mscratch::Register>,
    pub mepc: ReadWriteRiscvCsr<u32, mepc::mepc::Register>,
    pub mcause: ReadWriteRiscvCsr<u32, mcause::mcause::Register>,
    pub mtval: ReadWriteRiscvCsr<u32, mtval::mtval::Register>,
    pub mip: ReadWriteRiscvCsr<u32, mip::mip::Register>,
    pub mtvec: ReadWriteRiscvCsr<u32, mtvec::mtvec::Register>,
    pub stvec: ReadWriteRiscvCsr<u32, stvec::stvec::Register>,
    pub utvec: ReadWriteRiscvCsr<u32, utvec::utvec::Register>,
    pub mstatus: ReadWriteRiscvCsr<u32, mstatus::mstatus::Register>,
}

// Define the "addresses" of each CSR register.
pub const CSR: &CSR = &CSR {
    minstreth: ReadWriteRiscvCsr::new(riscv_csr::csr::MINSTRETH),
    minstret: ReadWriteRiscvCsr::new(riscv_csr::csr::MINSTRET),
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
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG1),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG2),
        ReadWriteRiscvCsr::new(riscv_csr::csr::PMPCFG3)
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
    ],
};

impl CSR {
    // resets the cycle counter to 0
    pub fn reset_cycle_counter(&self) {
        CSR.mcycleh.write(mcycle::mcycleh::mcycleh.val(0));
        CSR.mcycle.write(mcycle::mcycle::mcycle.val(0));
    }

    // reads the cycle counter
    pub fn read_cycle_counter(&self) -> u64 {
        let top = CSR.mcycleh.read(mcycle::mcycleh::mcycleh);
        let bot = CSR.mcycle.read(mcycle::mcycle::mcycle);

        u64::from(top).checked_shl(32).unwrap() + u64::from(bot)
    }
}
