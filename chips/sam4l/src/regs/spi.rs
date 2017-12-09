use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};

#[repr(C, packed)]
pub struct Registers {
    pub cr: WriteOnly<u32, Control::Register>,
    pub mr: ReadWrite<u32, Mode::Register>,
    pub rdr: ReadOnly<u32>,
    pub tdr: WriteOnly<u32, TransmitData::Register>,
    pub sr: ReadOnly<u32, Status::Register>,
    pub ier: WriteOnly<u32, InterruptFlags::Register>,
    pub idr: WriteOnly<u32, InterruptFlags::Register>,
    pub imr: ReadOnly<u32, InterruptFlags::Register>,
    _reserved0: [ReadOnly<u32>; 4],
    pub csr: [ReadWrite<u32, ChipSelectParams::Register>; 4],
    _reserved1: [ReadOnly<u32>; 41],
    pub wpcr: ReadWrite<u32, WriteProtectionControl::Register>,
    pub wpsr: ReadOnly<u32>,
    _reserved2: [ReadOnly<u32>; 3],
    pub features: ReadOnly<u32>,
    pub version: ReadOnly<u32>,
}

register_bitfields![u32,
    Control [
        LASTXFER 24,
        FLUSHFIFO 8,
        SWRST 7,
        SPIDIS 1,
        SPIEN 0
    ],

    Mode [
        DLYBCS   OFFSET(24)  NUMBITS(8),
        PCS      OFFSET(16)  NUMBITS(4),
        LLB      OFFSET( 7)  NUMBITS(1),
        RXFIFOEN OFFSET( 6)  NUMBITS(1),
        MODFDIS  OFFSET( 4)  NUMBITS(1),
        PCSDEC   OFFSET( 2)  NUMBITS(1),
        PS       OFFSET( 1)  NUMBITS(1),
        MSTR     OFFSET( 0)  NUMBITS(1)
    ],

    TransmitData [
        LASTXFER OFFSET(24)  NUMBITS(1),
        PCS      OFFSET(16)  NUMBITS(4),
        TD       OFFSET(0)   NUMBITS(16)
    ],

    Status [
        SPIENS  OFFSET(16),
        UNDES   OFFSET(10),
        TXEMPTY OFFSET(9),
        NSSR    OFFSET(8),
        OVRES   OFFSET(3),
        MODF    OFFSET(2),
        TDRE    OFFSET(1),
        RDRF    OFFSET(0)
    ],

    InterruptFlags [
        UNDES 10,
        TXEMPTY 9,
        NSSR 8,
        OVRES 3,
        MODF 2,
        TDRE 1,
        RDRF 0
    ],

    ChipSelectParams [
        DLYBCT OFFSET(24)  NUMBITS(8) [],
        DLYBS  OFFSET(16)  NUMBITS(8) [],
        SCBR   OFFSET(8)   NUMBITS(8) [],
        BITS   OFFSET(4)   NUMBITS(8) [
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
        CSAAT OFFSET(3)  NUMBITS(1) [
            ActiveAfterTransfer = 1,
            InactiveAfterTransfer = 0
        ],
        CSNAAT OFFSET(2)  NUMBITS(1) [
            DoNotRiseBetweenTransfers = 0,
            RiseBetweenTransfers = 1
        ],
        NCPHA OFFSET(1)  NUMBITS(1) [
            CaptureLeading = 1,
            CaptureTrailing = 0
        ],
        CPOL OFFSET(0)  NUMBITS(1) [
            InactiveHigh = 1,
            InactiveLow = 0
        ]
    ],

    WriteProtectionControl [
        SPIWPKEY OFFSET(8) NUMBITS(24) [
            Key = 0x535049
        ],
        SPIWPEN OFFSET(0) NUMBITS(1) []
    ]
];
