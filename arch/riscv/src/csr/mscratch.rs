use crate::XLEN;
use kernel::common::registers::register_bitfields;

register_bitfields![usize,
    pub mscratch [
        scratch OFFSET(0) NUMBITS(XLEN) []
    ]
];
