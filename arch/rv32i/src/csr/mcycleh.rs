use kernel::common::registers::register_bitfields;

// myclceh is the higher 32 bits of the number of elapsed cycles
register_bitfields![u32,
mcycleh [
    mcycleh OFFSET(0) NUMBITS(32) []
]
];
