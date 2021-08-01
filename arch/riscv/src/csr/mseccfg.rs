use kernel::utilities::registers::register_bitfields;

// Default to 32 bit if compiling for debug/testing.
#[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
register_bitfields![usize,
    pub mseccfg [
        mml OFFSET(0) NUMBITS(1) [],
        mwmp OFFSET(1) NUMBITS(1) [],
        rlb OFFSET(2) NUMBITS(1) [],
    ]
];

#[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
register_bitfields![usize,
    pub mseccfgh [
        // This isn't a real entry, it just avoids compilation errors
        none OFFSET(0) NUMBITS(1) [],
    ]
];

#[cfg(target_arch = "riscv64")]
register_bitfields![usize,
    pub mseccfg [
        MML OFFSET(0) NUMBITS(1) [],
        MMWP OFFSET(1) NUMBITS(1) [],
        RLB OFFSET(2) NUMBITS(1) [],
    ]
];
