use common::regs::{ReadWrite, ReadOnly};

pub const MCG: *mut Registers = 0x4006_4000 as *mut Registers;

#[repr(C, packed)]
pub struct Registers {
    pub c1: ReadWrite<u8, Control1>,
    pub c2: ReadWrite<u8, Control2>,
    pub c3: ReadWrite<u8>,
    pub c4: ReadWrite<u8>,
    pub c5: ReadWrite<u8, Control5>,
    pub c6: ReadWrite<u8, Control6>,
    pub s: ReadOnly<u8, Status>,
    _reserved0: ReadOnly<u8>,
    pub sc: ReadWrite<u8>,
    _reserved1: ReadOnly<u8>,
    pub atcvh: ReadWrite<u8>,
    pub atcvl: ReadWrite<u8>,
    pub c7: ReadWrite<u8>,
    pub c8: ReadWrite<u8>,
    pub c9: ReadWrite<u8>,
    _reserved2: ReadOnly<u8>,
    pub c11: ReadWrite<u8>,
    pub c12: ReadWrite<u8>,
    pub s2: ReadOnly<u8>,
    pub t3: ReadWrite<u8>
}

bitfields![u8,
    C1 Control1 [
        CLKS (6, Mask(0b11)) [
            LockedLoop = 0,
            Internal = 1,
            External = 2
        ],
        FRDIV (4, Mask(0b11)) [
            Low1_High32 = 0,
            Low2_High64 = 1,
            Low4_High128 = 2,
            Low8_High256 = 3,
            Low16_High512 = 4,
            Low32_High1024 = 5,
            Low64_High1280 = 6,
            Low128_High1536 = 7
        ],
        IREFS 2 [
            External = 0,
            SlowInternal = 1
        ],
        IRCLKEN 1 [
            Inactive = 0,
            Active = 1
        ],
        IREFSTEN 0 [
            IrefDisabledInStop = 0,
            IrefEnabledInStop = 1
        ]
    ],

    C2 Control2 [
        LOCKRE0 7 [],
        FCFTRIM 6 [],
        RANGE (4, Mask(0b11)) [
            Low = 0,
            High = 1,
            VeryHigh = 2
        ],
        HGO 3 [
            LowPower = 0,
            HighGain = 1
        ],
        EREFS 2 [
            External = 0,
            Oscillator = 1
        ],
        LP 1 [],
        IRCS 0 [
            SlowInternal = 0,
            FastInternal = 1
        ]
    ],

    C5 Control5 [
        PLLCLKEN 6 [],
        PLLSTEN 5 [],
        PRDIV (0, Mask(0b111)) [
            Div1 = 0, Div2 = 1, Div3 = 2, Div4 = 3,
            Div5 = 4, Div6 = 5, Div7 = 6, Div8 = 7
        ]
    ],

    C6 Control6 [
        LOLIE0 7 [],
        PLLS 6 [
            Fll = 0,
            PllcsOutput = 1
        ],
        CME0 5 [],
        VDIV (0, Mask(0b11111)) [
            Mul16 = 0, Mul17 = 1, Mul18 = 2, Mul19 = 3,
            Mul20 = 4, Mul21 = 5, Mul22 = 6, Mul23 = 7,
            Mul24 = 8, Mul25 = 9, Mul26 = 10, Mul27 = 11,
            Mul28 = 12, Mul29 = 13, Mul30 = 14, Mul31 = 15,
            Mul32 = 16, Mul33 = 17, Mul34 = 18, Mul35 = 19,
            Mul36 = 20, Mul37 = 21, Mul38 = 22, Mul39 = 23,
            Mul40 = 24, Mul41 = 25, Mul42 = 26, Mul43 = 27,
            Mul44 = 28, Mul45 = 29, Mul46 = 30, Mul47 = 31
        ]
    ],

    S Status [
        LOLS0 7 [],
        LOCK0 6 [],
        PLLST 5 [
            Fll = 0,
            PllcsOutput = 1
        ],
        IREFST 4 [
            External = 0,
            Internal = 1
        ],
        CLKST (2, Mask(0b11)) [
            Fll = 0,
            Internal = 1,
            External = 2,
            Pll = 3
        ],
        OSCINIT0 1 [],
        IRCST 0 [
            Slow = 0,
            Fast = 1
        ]
    ]
];
