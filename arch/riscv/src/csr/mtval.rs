use kernel::utilities::registers::register_bitfields;

// mtval contains the address of an exception
register_bitfields![usize,
    pub mtval [
        exception_addr OFFSET(0) NUMBITS(crate::XLEN) []
    ]
];
