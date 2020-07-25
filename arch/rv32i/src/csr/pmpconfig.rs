use kernel::common::registers::register_bitfields;

#[cfg(not(all(feature = "riscv64", target_os = "none")))]
register_bitfields![usize,
    pub pmpcfg [
        pmp0cfg OFFSET(0) NUMBITS(8) [],
        pmp1cfg OFFSET(8) NUMBITS(8) [],
        pmp2cfg OFFSET(16) NUMBITS(8) [],
        pmp3cfg OFFSET(24) NUMBITS(8) []
    ]
];

#[cfg(feature = "riscv64")]
register_bitfields![usize,
    pub pmpcfg [
        pmp0cfg OFFSET(0) NUMBITS(8) [],
        pmp1cfg OFFSET(8) NUMBITS(8) [],
        pmp2cfg OFFSET(16) NUMBITS(8) [],
        pmp3cfg OFFSET(24) NUMBITS(8) [],
        pmp4cfg OFFSET(32) NUMBITS(8) [],
        pmp5cfg OFFSET(40) NUMBITS(8) [],
        pmp6cfg OFFSET(48) NUMBITS(8) [],
        pmp7cfg OFFSET(56) NUMBITS(8) []
    ]
];
