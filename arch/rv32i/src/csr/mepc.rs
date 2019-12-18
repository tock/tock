use kernel::common::registers::register_bitfields;

// mepc contains address of instruction where trap occurred
register_bitfields![u32,
    pub mepc [
        trap_addr OFFSET(0) NUMBITS(32) []
    ]
];
