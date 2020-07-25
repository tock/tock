use crate::XLEN;
use kernel::common::registers::register_bitfields;

#[cfg(not(all(feature = "riscv64", target_os = "none")))]
register_bitfields![usize,
    pub pmpaddr [
        addr OFFSET(0) NUMBITS(XLEN) []
    ]
];

#[cfg(feature = "riscv64")]
register_bitfields![usize,
    pub pmpaddr [
        addr OFFSET(0) NUMBITS(XLEN - 10) []
    ]
];
