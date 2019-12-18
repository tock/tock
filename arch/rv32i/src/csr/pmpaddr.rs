use kernel::common::registers::register_bitfields;

register_bitfields![u32,
    pub pmpaddr [
        addr OFFSET(0) NUMBITS(32) []
    ]
];
