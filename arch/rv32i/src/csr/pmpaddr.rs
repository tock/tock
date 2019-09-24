use kernel::common::registers::register_bitfields;

register_bitfields![u32,
pmpaddr [
    addr OFFSET(0) NUMBITS(32) []
]
];
