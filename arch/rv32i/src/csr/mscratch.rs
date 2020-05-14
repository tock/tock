use kernel::common::registers::register_bitfields;

register_bitfields![u32,
    pub mscratch [
        scratch OFFSET(0) NUMBITS(32) []
    ]
];
