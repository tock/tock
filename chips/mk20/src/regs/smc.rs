use common::regs::{ReadWrite, ReadOnly};

#[repr(C, packed)]
pub struct Registers {
    pub pmprot: ReadWrite<u8, PowerModeProtection>,
    pub pmctrl: ReadWrite<u8, PowerModeControl>,
    pub stopctrl: ReadWrite<u8, StopControl>,
    pub pmstat: ReadOnly<u8, PowerModeStatus>
}

bitfields! [u8,
    PMPROT PowerModeProtection [
        AHSRUN 7 [],
        AVLP 5 [],
        ALLS 3 [],
        AVLLS 1 []
    ],
    PMCTRL PowerModeControl [
        RUNM (5, Mask(0b11)) [
            NormalRun = 0,
            VeryLowPowerRun = 2,
            HighSpeedRun = 3
        ],
        STOPA 3 [],
        STOPM (0, Mask(0b111)) [
            NormalStop = 0,
            VeryLowPowerStop = 2,
            LowLeakageStop = 3,
            VeryLowLeakageStop = 4
        ]
    ],
    STOPCTRL StopControl [
        PSTOPO (6, Mask(0b11)) [
            NormalStopMode = 0,
            PartialStop1 = 1,
            PartialStop2 = 2
        ],
        PORPO 5 [
            PORDetectEnabledInVLLS0 = 0,
            PORDetectDisabledInVLLS0 = 1
        ],
        RAM2PO 4 [
            RAM2NotPoweredInLLS2OrVLLS2 = 0,
            RAM2PoweredInLLS2AndVLLS2 = 1
        ],
        LLSM (0, Mask(0b111)) [
            EnterVLLS0 = 0,
            EnterVLLS1 = 1,
            EnterVLLS2OrLLS2 = 2,
            EnterVLLS3OrLLS3 = 3
        ]
    ],
    PMSTAT PowerModeStatus [
        PMSTAT (0, Mask(0xFF)) [
            Run = 1,
            Stop = 1<<1,
            VLPR = 1<<2,
            VLPW = 1<<3,
            VLPS = 1<<4,
            LLS = 1<<5,
            VLLS = 1<<6,
            HSRUN = 1<<7
        ]
    ]
];

pub const SMC_BASE: *mut Registers = 0x4007_E000 as *mut Registers;
