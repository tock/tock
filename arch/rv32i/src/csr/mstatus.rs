use kernel::common::registers::register_bitfields;

register_bitfields![usize,
    pub mstatus [
        uie OFFSET(0) NUMBITS(1) [],
        sie OFFSET(1) NUMBITS(1) [],
        mie OFFSET(3) NUMBITS(1) [],
        upie OFFSET(4) NUMBITS(1) [],
        spie OFFSET(5) NUMBITS(1) [],
        mpie OFFSET(7) NUMBITS(1) [],
        spp OFFSET(8) NUMBITS(1) [],
        mpp OFFSET(11) NUMBITS(2) [
            USER = 0,
            SUPERVISOR = 1,
            RESERVED = 2,
            MACHINE = 3
        ]
    ]
];
