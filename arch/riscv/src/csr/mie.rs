use kernel::utilities::registers::register_bitfields;

// mtvec contains the address(es) of the trap handler
register_bitfields![usize,
    pub mie [
        usoft OFFSET(0) NUMBITS(1) [],
        ssoft OFFSET(1) NUMBITS(1) [],
        msoft OFFSET(3) NUMBITS(1) [],
        utimer OFFSET(4) NUMBITS(1) [],
        stimer OFFSET(5) NUMBITS(1) [],
        mtimer OFFSET(7) NUMBITS(1) [],
        uext OFFSET(8) NUMBITS(1) [],
        sext OFFSET(9) NUMBITS(1) [],
        mext OFFSET(11) NUMBITS(1) [],
        BIT16 OFFSET(16) NUMBITS(1) [],
        BIT17 OFFSET(17) NUMBITS(1) [],
        BIT18 OFFSET(18) NUMBITS(1) [],
        BIT19 OFFSET(19) NUMBITS(1) [],
        BIT20 OFFSET(20) NUMBITS(1) [],
        BIT21 OFFSET(21) NUMBITS(1) [],
        BIT22 OFFSET(22) NUMBITS(1) [],
        BIT23 OFFSET(23) NUMBITS(1) [],
        BIT24 OFFSET(24) NUMBITS(1) [],
        BIT25 OFFSET(25) NUMBITS(1) [],
        BIT26 OFFSET(26) NUMBITS(1) [],
        BIT27 OFFSET(27) NUMBITS(1) [],
        BIT28 OFFSET(28) NUMBITS(1) [],
        BIT29 OFFSET(29) NUMBITS(1) [],
        BIT30 OFFSET(30) NUMBITS(1) [],
        BIT31 OFFSET(31) NUMBITS(1) [],
    ]
];
