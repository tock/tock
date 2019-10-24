use kernel::common::registers::register_bitfields;

// minstret is the lower 32 bits of the number of elasped instructions
register_bitfields![u32,
minstret [
    minstret OFFSET(0) NUMBITS(32) []
]
];
