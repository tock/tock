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
    minstreth: ReadWriteRiscvCsr::new(0xB82),
    minstret: ReadWriteRiscvCsr::new(0xB02),
    mcycleh: ReadWriteRiscvCsr::new(0xB80),
    mcycle: ReadWriteRiscvCsr::new(0xB00),
    mie: ReadWriteRiscvCsr::new(0x304),
    mtvec: ReadWriteRiscvCsr::new(0x305),
    mstatus: ReadWriteRiscvCsr::new(0x300),
    utvec: ReadWriteRiscvCsr::new(0x005),
    stvec: ReadWriteRiscvCsr::new(0x105),
    mscratch: ReadWriteRiscvCsr::new(0x340),
    mepc: ReadWriteRiscvCsr::new(0x341),
    mcause: ReadWriteRiscvCsr::new(0x342),
    mtval: ReadWriteRiscvCsr::new(0x343),
    mip: ReadWriteRiscvCsr::new(0x344),
    pmpcfg: [ReadWriteRiscvCsr::new(0x3A0); 4],
    pmpaddr: [ReadWriteRiscvCsr::new(0x3B0); 16],
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
