use kernel::common::registers::{register_bitfields, LocalRegisterCopy};

// utvec contains the address(es) of the trap handler
register_bitfields![u32,
    pub utvec [
        trap_addr OFFSET(2) NUMBITS(30) [],
        mode OFFSET(0) NUMBITS(2) [
            Direct = 0,
            Vectored = 1
        ]
    ]
];

trait UtvecHelpers {
    fn get_trap_address(&self) -> u32;
}

impl UtvecHelpers for LocalRegisterCopy<u32, utvec::Register> {
    fn get_trap_address(&self) -> u32 {
        self.read(utvec::trap_addr) << 2
    }
}
