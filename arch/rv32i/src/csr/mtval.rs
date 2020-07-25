use crate::XLEN;
use kernel::common::registers::register_bitfields;

// mtval contains the address of an exception
register_bitfields![usize,
    pub mtval [
        exception_addr OFFSET(0) NUMBITS(XLEN) []
    ]
];
