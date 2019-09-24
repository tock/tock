use kernel::common::registers::register_bitfields;

// mtval contains the address of an exception
register_bitfields![u32,
mtval [
    exception_addr OFFSET(0) NUMBITS(32) []
]
];
