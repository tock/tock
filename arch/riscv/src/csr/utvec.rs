use crate::XLEN;
use kernel::common::registers::{register_bitfields, LocalRegisterCopy};

// utvec contains the address(es) of the trap handler
register_bitfields![usize,
    pub utvec [
        trap_addr OFFSET(2) NUMBITS(XLEN - 2) [],
        mode OFFSET(0) NUMBITS(2) [
            Direct = 0,
            Vectored = 1
        ]
    ]
];

trait UtvecHelpers {
    fn get_trap_address(&self) -> usize;
}

impl UtvecHelpers for LocalRegisterCopy<usize, utvec::Register> {
    fn get_trap_address(&self) -> usize {
        self.read(utvec::trap_addr) << 2
    }
}
