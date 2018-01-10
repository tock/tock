use common::regs::{ReadWrite, ReadOnly};

#[repr(C, packed)]
pub struct Registers {
    pub mcr: ReadWrite<u32, ModuleControl>,
    _reserved0: [ReadOnly<u32>; 55],
    pub ltmr64h: ReadOnly<u32>,
    pub ltmr64l: ReadOnly<u32>,
    _reserved1: [ReadOnly<u32>; 2],
    pub timers: [PitRegisters; 4]
}

#[repr(C, packed)]
pub struct PitRegisters {
    pub ldval: ReadWrite<u32>, 
    pub cval: ReadOnly<u32>,
    pub tctrl: ReadWrite<u32, TimerControl>,
    pub tflg: ReadWrite<u32, TimerFlag>
}

bitfields! [u32,
    MCR ModuleControl [
        MDIS 1,
        FRZ 0
    ],
    TCTRL TimerControl [
        CHN 2,
        TIE 1,
        TEN 0
    ],
    TFLG TimerFlag [
        TIF 0
    ]
];

pub const PIT_BASE: *mut Registers = 0x4003_7000 as *mut Registers;
pub const PIT_ADDRS: [*mut PitRegisters; 4] = [0x4003_7100 as *mut PitRegisters,
                                               0x4003_7110 as *mut PitRegisters,
                                               0x4003_7120 as *mut PitRegisters,
                                               0x4003_7130 as *mut PitRegisters];
