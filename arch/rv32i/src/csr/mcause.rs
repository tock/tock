use kernel::common::registers::{register_bitfields, LocalRegisterCopy};

register_bitfields![u32,
mcause [
    // only support 32 bit targets
    isException OFFSET(31) NUMBITS(1) [],
    reason OFFSET(0) NUMBITS(31) []
]
];

pub trait McauseHelpers {
    fn is_interrupt(&self) -> bool;
    fn cause(&self) -> Trap;
}

impl McauseHelpers for LocalRegisterCopy<u32, mcause::Register> {
    fn is_interrupt(&self) -> bool {
        self.read(mcause::isException).eq(&1)
    }
    fn cause(&self) -> Trap {
        if self.is_interrupt() {
            Trap::Interrupt(Interrupt::from(self.read(mcause::reason)))
        } else {
            Trap::Exception(Exception::from(self.read(mcause::reason)))
        }
    }
}

/// Trap Cause
#[derive(Copy, Clone, Debug)]
pub enum Trap {
    Interrupt(Interrupt),
    Exception(Exception),
}

/// Interrupt
#[derive(Copy, Clone, Debug)]
pub enum Interrupt {
    UserSoft,
    SupervisorSoft,
    MachineSoft,
    UserTimer,
    SupervisorTimer,
    MachineTimer,
    UserExternal,
    SupervisorExternal,
    MachineExternal,
    Unknown,
}

/// Exception
#[derive(Copy, Clone, Debug)]
pub enum Exception {
    InstructionMisaligned,
    InstructionFault,
    IllegalInstruction,
    Breakpoint,
    LoadMisaligned,
    LoadFault,
    StoreMisaligned,
    StoreFault,
    UserEnvCall,
    SupervisorEnvCall,
    MachineEnvCall,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
    Unknown,
}

impl Interrupt {
    pub fn from(nr: u32) -> Self {
        match nr {
            0 => Interrupt::UserSoft,
            1 => Interrupt::SupervisorSoft,
            3 => Interrupt::MachineSoft,
            4 => Interrupt::UserTimer,
            5 => Interrupt::SupervisorTimer,
            7 => Interrupt::MachineTimer,
            8 => Interrupt::UserExternal,
            9 => Interrupt::SupervisorExternal,
            11 => Interrupt::MachineExternal,
            _ => Interrupt::Unknown,
        }
    }
}

impl Exception {
    pub fn from(nr: u32) -> Self {
        match nr {
            0 => Exception::InstructionMisaligned,
            1 => Exception::InstructionFault,
            2 => Exception::IllegalInstruction,
            3 => Exception::Breakpoint,
            4 => Exception::LoadMisaligned,
            5 => Exception::LoadFault,
            6 => Exception::StoreMisaligned,
            7 => Exception::StoreFault,
            8 => Exception::UserEnvCall,
            9 => Exception::SupervisorEnvCall,
            11 => Exception::MachineEnvCall,
            12 => Exception::InstructionPageFault,
            13 => Exception::LoadPageFault,
            15 => Exception::StorePageFault,
            _ => Exception::Unknown,
        }
    }
}
