use kernel::common::regs::{ReadWrite, ReadOnly, WriteOnly};

#[repr(C, packed)]
pub struct Registers {
    pub cr: WriteOnly<u32, Control>,
    pub mr: ReadWrite<u32, Mode>,
    pub rdr: ReadOnly<u32>,
    pub tdr: WriteOnly<u32, TransmitData>,
    pub sr: ReadOnly<u32, Status>,
    pub ier: WriteOnly<u32, InterruptFlags>,
    pub idr: WriteOnly<u32, InterruptFlags>,
    pub imr: ReadOnly<u32, InterruptFlags>,
    _reserved0: [ReadOnly<u32>; 4],
    pub csr: [ReadWrite<u32, ChipSelectParams>; 4],
    _reserved1: [ReadOnly<u32>; 41],
    pub wpcr: ReadWrite<u32, WriteProtectionControl>,
    pub wpsr: ReadOnly<u32>,
    _reserved2: [ReadOnly<u32>; 3],
    pub features: ReadOnly<u32>,
    pub version: ReadOnly<u32>,
}

bitfields![u32,
    CR Control [
        LASTXFER 24 [],
        FLUSHFIFO 8 [],
        SWRST 7 [],
        SPIDIS 1 [],
        SPIEN 0 []
    ],

    MR Mode [
        DLYBCS (24, Mask(0xFF)) [],
        PCS (16, Mask(0xF)) [],
        LLB 7 [],
        RXFIFOEN 6 [],
        MODFDIS 4 [],
        PCSDEC 2 [],
        PS 1 [],
        MSTR 0 []
    ],

    TDR TransmitData [
        LASTXFER 24 [],
        PCS (16, Mask(0xF)) [],
        TD (0, Mask(0xFFFF)) []
    ],

    SR Status [
        SPIENS 16 [],
        UNDES 10 [],
        TXEMPTY 9 [],
        NSSR 8 [],
        OVRES 3 [],
        MODF 2 [],
        TDRE 1 [],
        RDRF 0 []
    ],

    INT InterruptFlags [
        UNDES 10 [],
        TXEMPTY 9 [],
        NSSR 8 [],
        OVRES 3 [],
        MODF 2 [],
        TDRE 1 [],
        RDRF 0 []
    ],

    CSR ChipSelectParams [
        DLYBCT (24, Mask(0xFF)) [],
        DLYBS (16, Mask(0xFF)) [],
        SCBR (8, Mask(0xFF)) [],
        BITS (4, Mask(0xF)) [
            Eight = 0,
            Nine = 1,
            Ten = 2,
            Eleven = 3,
            Twelve = 4,
            Thirteen = 5,
            Fourteen = 6,
            Fifteen = 7,
            Sixteen = 8,
            Four = 9,
            Five = 10,
            Six = 11,
            Seven = 12
        ],
        CSAAT 3 [
            ActiveAfterTransfer = 1,
            InactiveAfterTransfer = 0
        ],
        CSNAAT 2 [
            DoNotRiseBetweenTransfers = 0,
            RiseBetweenTransfers = 1
        ],
        NCPHA 1 [
            CaptureLeading = 1,
            CaptureTrailing = 0
        ],
        CPOL 0 [
            InactiveHigh = 1,
            InactiveLow = 0
        ]
    ],

    WPCR WriteProtectionControl [
        SPIWPKEY (8, Mask(0xFFFFFF)) [
            Key = 0x535049
        ],
        SPIWPEN 0 []
    ]
];
