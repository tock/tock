use kernel::common::registers::register_bitfields;

// mepc contains address of instruction where trap occurred
register_bitfields![u32,
mepc [
    trap_addr OFFSET(0) NUMBITS(32) []
]
];
