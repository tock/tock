use common::regs::{ReadWrite, ReadOnly};

#[repr(C, packed)]
pub struct Registers {
    pub bdh: ReadWrite<u8, BaudRateHigh>,
    pub bdl: ReadWrite<u8>,
    pub c1: ReadWrite<u8, Control1>,
    pub c2: ReadWrite<u8, Control2>,
    pub s1: ReadOnly<u8, Status1>,
    pub s2: ReadWrite<u8, Status2>,
    pub c3: ReadWrite<u8, Control3>,
    pub d: ReadWrite<u8>,
    pub ma1: ReadWrite<u8>,
    pub ma2: ReadWrite<u8>,
    pub c4: ReadWrite<u8, Control4>,
    pub c5: ReadWrite<u8, Control5>,
    pub ed: ReadOnly<u8>,
    pub modem: ReadWrite<u8>,
    pub ir: ReadWrite<u8>, // 0x0E
    _reserved0: ReadWrite<u8>,
    pub pfifo: ReadWrite<u8>, // 0x10
    pub cfifo: ReadWrite<u8>,
    pub sfifo: ReadWrite<u8>,
    pub twfifo: ReadWrite<u8>,
    pub tcfifo: ReadOnly<u8>,
    pub rwfifo: ReadWrite<u8>,
    pub rcfifo: ReadOnly<u8>, // 0x16
    _reserved1: ReadWrite<u8>,
    pub c7816: ReadWrite<u8>, // 0x18
    pub ie7816: ReadWrite<u8>,
    pub is7816: ReadWrite<u8>,
    pub wp7816: ReadWrite<u8>,
    pub wn7816: ReadWrite<u8>,
    pub wf7816: ReadWrite<u8>,
    pub et7816: ReadWrite<u8>,
    pub tl7816: ReadWrite<u8>, // 0x1F
    _reserved2: [ReadWrite<u8>; 26],
    pub ap7816a_t0: ReadWrite<u8>, // 0x3A
    pub ap7816b_t0: ReadWrite<u8>,
    pub wp7816a_t0_t1: ReadWrite<u8>,
    pub wp7816b_t0_t1: ReadWrite<u8>,
    pub wgp7816_t1: ReadWrite<u8>,
    pub wp7816c_t1: ReadWrite<u8>,
}

pub const UART_BASE_ADDRS: [*mut Registers; 5] = [0x4006A000 as *mut Registers,
                                                  0x4006B000 as *mut Registers,
                                                  0x4006C000 as *mut Registers,
                                                  0x4006D000 as *mut Registers,
                                                  0x400EA000 as *mut Registers];

bitfields! {u8,
    BDH BaudRateHigh [
        LBKDIE 7 [],
        RXEDGIE 6 [],
        SBNS 5 [
            One = 0,
            Two = 1
        ],
        SBR (0, Mask(0b11111)) []
    ],
    C1 Control1 [
        LOOPS 7 [],
        UARTSWAI 6 [],
        RSRC 5 [],
        M 4 [
            EightBit = 0,
            NineBit = 1
        ],
        WAKE 3 [
            Idle = 0,
            AddressMark = 1
        ],
        ILT 2 [
            AfterStart = 0,
            AfterStop = 1
        ],
        PE 1 [],
        PT 0 [
            Even = 0,
            Odd = 1
        ]
    ],
    C2 Control2 [
        TIE 7,
        TCIE 6,
        RIE 5,
        ILIE 4,
        TE 3,
        RE 2,
        RWU 1,
        SBK 0
    ],
    S1 Status1 [
        TRDE 7,
        TC 6,
        RDRF 5,
        IDLE 4,
        OR 3,
        NF 2,
        FE 1,
        PF 0
    ],
    S2 Status2 [
        LBKDIF 7,
        RXEDGIF 6,
        MSBF 5,
        RXINV 4,
        RWUID 3,
        BRK13 2,
        LBKDE 1,
        RAF 0
    ],
    C3 Control3 [
        R8 7,
        T8 6,
        TXDIR 5,
        TXINV 4,
        ORIE 3,
        NEIE 2,
        FEIE 1,
        PEIE 0
    ],
    C4 Control4 [
        MAEN1 7 [],
        MAEN2 6 [],
        M10 5 [],
        BRFA (0, Mask(0b11111)) []
    ],
    C5 Control5 [
        TDMAS 7,
        RDMAS 5
    ]
}
