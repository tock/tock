# RISC-V CSR Interface for Tock

This library is based on the Tock Register Interface to provide a consistent
interface for accessing the CSRs (Control and Status Registers) on RISC-V
platforms.


## Example Usage

```
use riscv_csr::csr::ReadWriteRiscvCsr;

// Bit definitions for the `mstatus` register. Need these for all registers.
register_bitfields![u32,
    mstatus [
        uie OFFSET(0) NUMBITS(1) [],
        sie OFFSET(1) NUMBITS(1) [],
        mie OFFSET(3) NUMBITS(1) [],
        upie OFFSET(4) NUMBITS(1) [],
        spie OFFSET(5) NUMBITS(1) [],
        mpie OFFSET(7) NUMBITS(1) [],
        spp OFFSET(8) NUMBITS(1) [],
        mpp OFFSET(11) NUMBITS(2) [
            USER = 0,
            SUPERVISOR = 1,
            RESERVED = 2,
            MACHINE = 3
        ]
    ]
]

/// Setup the struct of CSR register names.
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

// Define the address of all the CSRs.
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
```
