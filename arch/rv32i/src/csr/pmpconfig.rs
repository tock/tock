use kernel::common::registers::register_bitfields;

// pmpcfg

register_bitfields![u32,
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
