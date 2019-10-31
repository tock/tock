use kernel::common::registers::register_bitfields;

// myclce is the lower 32 bits of the number of elapsed cycles
register_bitfields![u32,
mcycle [
    mcycle OFFSET(0) NUMBITS(32) []
]
];

// myclceh is the higher 32 bits of the number of elapsed cycles
register_bitfields![u32,
mcycleh [
    mcycleh OFFSET(0) NUMBITS(32) []
]
];
