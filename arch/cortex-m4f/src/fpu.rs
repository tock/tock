//! ARM FPU Block

use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;

register_structs! {
    pub FpuRegisters {
        (0x00 => _reserved0),

        /// Floating-point Context Control Register
        (0x04 => fpccr: ReadWrite<u32, FPCCR::Register>),

        /// Floating-point Context Address Register
        (0x08 => fpcar: ReadWrite<u32, FPCAR::Register>),

        /// Floating-point Default Status Control Register
        (0x0C => fpscr: ReadWrite<u32, FPSCR::Register>),

        (0x10 => @END),
    }
}

register_bitfields![u32,
    FPCCR [
        ASPEN   OFFSET(31)  NUMBITS(1),
        LSPEN   OFFSET(30)  NUMBITS(1),
        MONRDY  OFFSET(8)   NUMBITS(1),
        BFRDY   OFFSET(6)   NUMBITS(1),
        MMRDY   OFFSET(5)   NUMBITS(1),
        HFRDY   OFFSET(4)   NUMBITS(1),
        THREAD  OFFSET(3)   NUMBITS(1),
        USER    OFFSET(1)   NUMBITS(1),
        LSPACT  OFFSET(0)   NUMBITS(1),
    ],

    FPCAR [
        ADDRESS OFFSET(3)   NUMBITS(29),
    ],

    FPSCR [
        AHP     OFFSET(26)  NUMBITS(1),
        DN      OFFSET(25)  NUMBITS(1),
        FZ      OFFSET(24)  NUMBITS(1),
        RMode   OFFSET(22)  NUMBITS(2),
    ],
];

pub const FPU: StaticRef<FpuRegisters> =
    unsafe { StaticRef::new(0xE000_EF30 as *const FpuRegisters) };

pub unsafe fn enable_auto_state_save() {
    FPU.fpccr.modify(FPCCR::ASPEN::SET);
}

pub unsafe fn enable_auto_lazy_store() {
    FPU.fpccr.modify(FPCCR::LSPEN::SET);
}
