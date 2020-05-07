// Flash Controller (FLCTL)

use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;

const FLCTL_BASE: StaticRef<FlCtlRegisters> =
    unsafe { StaticRef::new(4001_1000 as *const FlCtlRegisters) };

#[repr(C)]
struct FlCtlRegisters {
    power_stat: ReadOnly<u32, POWER_STAT::Register>,
    _reserved0: [u32; 3],
    bank0_rdctl: ReadWrite<u32, BANK0_RDCTL::Register>,
    bank1_rdctl: ReadWrite<u32, BANK1_RDCTL::Register>,
    _reserved1: [u32; 67], // most of this memory are registers, but for now they are not used
}

register_bitfields! [u32,
    POWER_STAT [
        PSTAT OFFSET(0) NUMBITS(3),
        LDOSTAT OFFSET(3) NUMBITS(1),
        VREFSTAT OFFSET(4) NUMBITS(1),
        IREFSTAT OFFSET(5) NUMBITS(1),
        TRIMSTAT OFFSET(6) NUMBITS(1),
        RD_2T OFFSET(7) NUMBITS(1)
    ],
    BANK0_RDCTL [
        RD_MODE OFFSET(0) NUMBITS(4),
        BUFI OFFSET(4) NUMBITS(1),
        BUFD OFFSET(5) NUMBITS(1),
        WAIT OFFSET(12) NUMBITS(4),
        RD_MODE_STATUS OFFSET(16) NUMBITS(4)
    ],
    BANK1_RDCTL [
        RD_MODE OFFSET(0) NUMBITS(4),
        BUFI OFFSET(4) NUMBITS(1),
        BUFD OFFSET(5) NUMBITS(1),
        WAIT OFFSET(12) NUMBITS(4),
        RD_MODE_STATUS OFFSET(16) NUMBITS(4)
    ]
];

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum WaitStates {
    _0 = 0,
    _1 = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    _6 = 6,
    _7 = 7,
    _8 = 8,
    _9 = 9,
    _10 = 10,
    _11 = 11,
    _12 = 12,
    _13 = 13,
    _14 = 14,
    _15 = 15,
}

pub struct FlCtl {
    registers: StaticRef<FlCtlRegisters>,
}

impl FlCtl {
    pub const fn new() -> FlCtl {
        FlCtl {
            registers: FLCTL_BASE,
        }
    }

    pub fn set_waitstates(&self, ws: WaitStates) {
        self.registers
            .bank0_rdctl
            .modify(BANK0_RDCTL::WAIT.val(ws as u32));
        self.registers
            .bank1_rdctl
            .modify(BANK1_RDCTL::WAIT.val(ws as u32));
    }

    pub fn set_buffering(&self, enable: bool) {
        self.registers
            .bank0_rdctl
            .modify(BANK0_RDCTL::BUFD.val(enable as u32) + BANK0_RDCTL::BUFI.val(enable as u32));
        self.registers
            .bank1_rdctl
            .modify(BANK1_RDCTL::BUFD.val(enable as u32) + BANK1_RDCTL::BUFI.val(enable as u32));
    }
}
