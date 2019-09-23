use kernel::common::registers::register_bitfields;

// mtvec contains the address(es) of the trap handler
register_bitfields![u32,
mie [
    usoft OFFSET(0) NUMBITS(1) [],
    ssoft OFFSET(1) NUMBITS(1) [],
    msoft OFFSET(2) NUMBITS(1) [],
    utimer OFFSET(3) NUMBITS(1) [],
    stimer OFFSET(4) NUMBITS(1) [],
    mtimer OFFSET(5) NUMBITS(1) [],
    uext OFFSET(6) NUMBITS(1) [],
    sext OFFSET(7) NUMBITS(1) [],
    mext OFFSET(8) NUMBITS(1) [],
    lie0 OFFSET(9) NUMBITS(1) [],
    lie1 OFFSET(10) NUMBITS(1) [],
    lie2 OFFSET(11) NUMBITS(1) [],
    lie3 OFFSET(12) NUMBITS(1) [],
    lie4 OFFSET(13) NUMBITS(1) [],
    lie5 OFFSET(14) NUMBITS(1) [],
    lie6 OFFSET(15) NUMBITS(1) [],
    lie7 OFFSET(16) NUMBITS(1) [],
    lie8 OFFSET(17) NUMBITS(1) [],
    lie9 OFFSET(18) NUMBITS(1) [],
    lie10 OFFSET(19) NUMBITS(1) [],
    lie11 OFFSET(20) NUMBITS(1) [],
    lie12 OFFSET(21) NUMBITS(1) [],
    lie13 OFFSET(22) NUMBITS(1) [],
    lie14 OFFSET(23) NUMBITS(1) [],
    lie15 OFFSET(24) NUMBITS(1) []
]
];
