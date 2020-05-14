use kernel::common::registers::{register_bitfields, LocalRegisterCopy};

// mtvec contains the address(es) of the trap handler
register_bitfields![u32,
    pub mtvec [
        trap_addr OFFSET(2) NUMBITS(30) [],
        mode OFFSET(0) NUMBITS(2) [
            Direct = 0,
            Vectored = 1
        ]
    ]
];

trait MtvecHelpers {
    fn get_trap_address(&self) -> u32;
}

impl MtvecHelpers for LocalRegisterCopy<u32, mtvec::Register> {
    fn get_trap_address(&self) -> u32 {
        self.read(mtvec::trap_addr) << 2
    }
}
