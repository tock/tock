use kernel::common::registers::register_bitfields;

// minstret is the lower 32 bits of the number of elasped instructions
register_bitfields![u32,
minstret [
    minstret OFFSET(0) NUMBITS(32) []
]
];

// minstreth is the higher 32 bits of the number of elapsed instructions
register_bitfields![u32,
minstreth [
    minstreth OFFSET(0) NUMBITS(32) []
]
];
