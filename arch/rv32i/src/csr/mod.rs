use riscv_csr::csr::ReadWriteRiscvCsr;

pub mod mcause;
pub mod mepc;
pub mod mie;
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
    pub pmpcfg0: ReadWriteRiscvCsr<u32, pmpconfig::pmpcfg::Register>,
    pub pmpcfg1: ReadWriteRiscvCsr<u32, pmpconfig::pmpcfg::Register>,
    pub pmpcfg2: ReadWriteRiscvCsr<u32, pmpconfig::pmpcfg::Register>,
    pub pmpcfg3: ReadWriteRiscvCsr<u32, pmpconfig::pmpcfg::Register>,
    pub pmpaddr0: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr1: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr2: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr3: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr4: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr5: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr6: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr7: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr8: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr9: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr10: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr11: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr12: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr13: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr14: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
    pub pmpaddr15: ReadWriteRiscvCsr<u32, pmpaddr::pmpaddr::Register>,
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
    pmpcfg0: ReadWriteRiscvCsr::new(0x3A0),
    pmpcfg1: ReadWriteRiscvCsr::new(0x3A1),
    pmpcfg2: ReadWriteRiscvCsr::new(0x3A2),
    pmpcfg3: ReadWriteRiscvCsr::new(0x3A3),
    pmpaddr0: ReadWriteRiscvCsr::new(0x3B0),
    pmpaddr1: ReadWriteRiscvCsr::new(0x3B1),
    pmpaddr2: ReadWriteRiscvCsr::new(0x3B2),
    pmpaddr3: ReadWriteRiscvCsr::new(0x3B3),
    pmpaddr4: ReadWriteRiscvCsr::new(0x3B4),
    pmpaddr5: ReadWriteRiscvCsr::new(0x3B5),
    pmpaddr6: ReadWriteRiscvCsr::new(0x3B6),
    pmpaddr7: ReadWriteRiscvCsr::new(0x3B7),
    pmpaddr8: ReadWriteRiscvCsr::new(0x3B8),
    pmpaddr9: ReadWriteRiscvCsr::new(0x3B9),
    pmpaddr10: ReadWriteRiscvCsr::new(0x3BA),
    pmpaddr11: ReadWriteRiscvCsr::new(0x3BB),
    pmpaddr12: ReadWriteRiscvCsr::new(0x3BC),
    pmpaddr13: ReadWriteRiscvCsr::new(0x3BD),
    pmpaddr14: ReadWriteRiscvCsr::new(0x3BE),
    pmpaddr15: ReadWriteRiscvCsr::new(0x3BF),
};
