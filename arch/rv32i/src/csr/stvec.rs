use crate::XLEN;
use kernel::common::registers::{register_bitfields, LocalRegisterCopy};

// stvec contains the address(es) of the trap handler
register_bitfields![usize,
    pub stvec [
        trap_addr OFFSET(2) NUMBITS(XLEN - 2) [],
        mode OFFSET(0) NUMBITS(2) [
            Direct = 0,
            Vectored = 1
        ]
    ]
];

trait StvecHelpers {
    fn get_trap_address(&self) -> usize;
}

impl StvecHelpers for LocalRegisterCopy<usize, stvec::Register> {
    fn get_trap_address(&self) -> usize {
        self.read(stvec::trap_addr) << 2
    }
}
