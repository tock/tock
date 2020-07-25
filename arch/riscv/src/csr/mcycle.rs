use crate::XLEN;
use kernel::common::registers::register_bitfields;

// mcycle is the lower XLEN bits of the number of elapsed cycles
register_bitfields![usize,
    pub mcycle [
        mcycle OFFSET(0) NUMBITS(XLEN) []
    ]
];

// mcycleh is the higher XLEN bits of the number of elapsed cycles
// it does not exist on riscv64
#[cfg(not(feature = "riscv64"))]
register_bitfields![usize,
    pub mcycleh [
        mcycleh OFFSET(0) NUMBITS(XLEN) []
    ]
];
