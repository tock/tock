use kernel::utilities::registers::register_bitfields;

register_bitfields![usize,
    pub mscratch [
        scratch OFFSET(0) NUMBITS(crate::XLEN) []
    ]
];
