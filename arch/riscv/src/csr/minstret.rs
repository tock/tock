use kernel::utilities::registers::register_bitfields;

// minstret is the lower XLEN bits of the number of elapsed instructions
register_bitfields![usize,
    pub minstret [
        minstret OFFSET(0) NUMBITS(crate::XLEN) []
    ]
];

// `minstreth` is the higher XLEN bits of the number of elapsed instructions.
// It does not exist on riscv64.
#[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
register_bitfields![usize,
    pub minstreth [
        minstreth OFFSET(0) NUMBITS(crate::XLEN) []
    ]
];
