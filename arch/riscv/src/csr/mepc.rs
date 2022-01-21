use kernel::utilities::registers::register_bitfields;

// mepc contains address of instruction where trap occurred
register_bitfields![usize,
    pub mepc [
        trap_addr OFFSET(0) NUMBITS(crate::XLEN) []
    ]
];
