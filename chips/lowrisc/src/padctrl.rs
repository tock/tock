//! General Purpose Input/Output driver.

use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};

register_structs! {
    pub PadCtrlRegisters {
        (0x00 => pub regen: ReadWrite<u32, REGEN::Register>),
        (0x04 => pub dio_pads: ReadWrite<u32, DIO_PADS::Register>),
        (0x08 => pub mio_pads0: ReadWrite<u32, DIO_PADS::Register>),
        (0x0c => pub mio_pads1: ReadWrite<u32, DIO_PADS::Register>),
        (0x10 => pub mio_pads2: ReadWrite<u32, DIO_PADS::Register>),
        (0x14 => pub mio_pads3: ReadWrite<u32, DIO_PADS::Register>),
        (0x18 => @END),
    }
}

register_bitfields![u32,
    pub REGEN [
        WEN OFFSET(0) NUMBITS(1) []
    ],
    pub DIO_PADS [
        ATTR0_IO_INV OFFSET(0) NUMBITS(1) [],
        ATTR0_OPEN_DRAIN OFFSET(1) NUMBITS(1) [],
        ATTR0_PULL_DOWN OFFSET(2) NUMBITS(1) [],
        ATTR0_PULL_UP OFFSET(3) NUMBITS(1) [],
        ATTR0_KEEPER OFFSET(4) NUMBITS(1) [],
        ATTR0_STRENGTH OFFSET(5) NUMBITS(1) [],
        ATTR1_IO_INV OFFSET(8) NUMBITS(1) [],
        ATTR1_OPEN_DRAIN OFFSET(9) NUMBITS(1) [],
        ATTR1_PULL_DOWN OFFSET(10) NUMBITS(1) [],
        ATTR1_PULL_UP OFFSET(11) NUMBITS(1) [],
        ATTR1_KEEPER OFFSET(12) NUMBITS(1) [],
        ATTR1_STRENGTH OFFSET(13) NUMBITS(1) [],
        ATTR2_IO_INV OFFSET(16) NUMBITS(1) [],
        ATTR2_OPEN_DRAIN OFFSET(17) NUMBITS(1) [],
        ATTR2_PULL_DOWN OFFSET(18) NUMBITS(1) [],
        ATTR2_PULL_UP OFFSET(19) NUMBITS(1) [],
        ATTR2_KEEPER OFFSET(20) NUMBITS(1) [],
        ATTR2_STRENGTH OFFSET(21) NUMBITS(1) [],
        ATTR3_IO_INV OFFSET(24) NUMBITS(1) [],
        ATTR3_OPEN_DRAIN OFFSET(25) NUMBITS(1) [],
        ATTR3_PULL_DOWN OFFSET(26) NUMBITS(1) [],
        ATTR3_PULL_UP OFFSET(27) NUMBITS(1) [],
        ATTR3_KEEPER OFFSET(28) NUMBITS(1) [],
        ATTR3_STRENGTH OFFSET(29) NUMBITS(1) []
    ]
];
