use kernel::common::registers::register_bitfields;

// mtvec contains the address(es) of the trap handler
register_bitfields![u32,
mie [
    usoft OFFSET(0) NUMBITS(1) [],
    ssoft OFFSET(1) NUMBITS(1) [],
    msoft OFFSET(3) NUMBITS(1) [],
    utimer OFFSET(4) NUMBITS(1) [],
    stimer OFFSET(5) NUMBITS(1) [],
    mtimer OFFSET(7) NUMBITS(1) [],
    uext OFFSET(8) NUMBITS(1) [],
    sext OFFSET(9) NUMBITS(1) [],
    mext OFFSET(11) NUMBITS(1) []
]
];
