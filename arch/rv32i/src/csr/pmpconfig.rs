use kernel::common::registers::register_bitfields;

// Default to 32 bit if compiling for debug/testing.
#[cfg(any(target_arch = "riscv32", not(target_os = "none")))]
register_bitfields![usize,
    pub pmpcfg [
        r0 OFFSET(0) NUMBITS(1) [],
        w0 OFFSET(1) NUMBITS(1) [],
        x0 OFFSET(2) NUMBITS(1) [],
        a0 OFFSET(3) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l0 OFFSET(7) NUMBITS(1) [],

        r1 OFFSET(8) NUMBITS(1) [],
        w1 OFFSET(9) NUMBITS(1) [],
        x1 OFFSET(10) NUMBITS(1) [],
        a1 OFFSET(11) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l1 OFFSET(15) NUMBITS(1) [],

        r2 OFFSET(16) NUMBITS(1) [],
        w2 OFFSET(17) NUMBITS(1) [],
        x2 OFFSET(18) NUMBITS(1) [],
        a2 OFFSET(19) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l2 OFFSET(23) NUMBITS(1) [],

        r3 OFFSET(24) NUMBITS(1) [],
        w3 OFFSET(25) NUMBITS(1) [],
        x3 OFFSET(26) NUMBITS(1) [],
        a3 OFFSET(27) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l3 OFFSET(31) NUMBITS(1) []
    ]
];

#[cfg(target_arch = "riscv64")]
register_bitfields![usize,
    pub pmpcfg [
        r0 OFFSET(0) NUMBITS(1) [],
        w0 OFFSET(1) NUMBITS(1) [],
        x0 OFFSET(2) NUMBITS(1) [],
        a0 OFFSET(3) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l0 OFFSET(7) NUMBITS(1) [],

        r1 OFFSET(8) NUMBITS(1) [],
        w1 OFFSET(9) NUMBITS(1) [],
        x1 OFFSET(10) NUMBITS(1) [],
        a1 OFFSET(11) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l1 OFFSET(15) NUMBITS(1) [],

        r2 OFFSET(16) NUMBITS(1) [],
        w2 OFFSET(17) NUMBITS(1) [],
        x2 OFFSET(18) NUMBITS(1) [],
        a2 OFFSET(19) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l2 OFFSET(23) NUMBITS(1) [],

        r3 OFFSET(24) NUMBITS(1) [],
        w3 OFFSET(25) NUMBITS(1) [],
        x3 OFFSET(26) NUMBITS(1) [],
        a3 OFFSET(27) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l3 OFFSET(31) NUMBITS(1) [],

        r4 OFFSET(32) NUMBITS(1) [],
        w4 OFFSET(33) NUMBITS(1) [],
        x4 OFFSET(34) NUMBITS(1) [],
        a4 OFFSET(35) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l4 OFFSET(39) NUMBITS(1) [],

        r5 OFFSET(40) NUMBITS(1) [],
        w5 OFFSET(41) NUMBITS(1) [],
        x5 OFFSET(42) NUMBITS(1) [],
        a5 OFFSET(43) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l5 OFFSET(47) NUMBITS(1) [],

        r6 OFFSET(48) NUMBITS(1) [],
        w6 OFFSET(49) NUMBITS(1) [],
        x6 OFFSET(50) NUMBITS(1) [],
        a6 OFFSET(51) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l6 OFFSET(55) NUMBITS(1) [],

        r7 OFFSET(56) NUMBITS(1) [],
        w7 OFFSET(57) NUMBITS(1) [],
        x7 OFFSET(58) NUMBITS(1) [],
        a7 OFFSET(59) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l7 OFFSET(63) NUMBITS(1) []
    ]
];
