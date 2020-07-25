use crate::XLEN;
use kernel::common::registers::register_bitfields;

// minstret is the lower XLEN bits of the number of elapsed instructions
register_bitfields![usize,
    pub minstret [
        minstret OFFSET(0) NUMBITS(XLEN) []
    ]
];

// minstreth is the higher XLEN bits of the number of elapsed instructions
// it does not exist on riscv64
#[cfg(not(feature = "riscv64"))]
register_bitfields![usize,
    pub minstreth [
        minstreth OFFSET(0) NUMBITS(XLEN) []
    ]
];
