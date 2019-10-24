use kernel::common::registers::register_bitfields;

// minstreth is the higher 32 bits of the number of elapsed instructions
register_bitfields![u32,
minstreth [
    minstreth OFFSET(0) NUMBITS(32) []
]
];
