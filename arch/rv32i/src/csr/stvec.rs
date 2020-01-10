use kernel::common::registers::{register_bitfields, LocalRegisterCopy};

// stvec contains the address(es) of the trap handler
register_bitfields![u32,
    pub stvec [
        trap_addr OFFSET(2) NUMBITS(30) [],
        mode OFFSET(0) NUMBITS(2) [
            Direct = 0,
            Vectored = 1
        ]
    ]
];

trait StvecHelpers {
    fn get_trap_address(&self) -> u32;
}

impl StvecHelpers for LocalRegisterCopy<u32, stvec::Register> {
    fn get_trap_address(&self) -> u32 {
        self.read(stvec::trap_addr) << 2
    }
}
