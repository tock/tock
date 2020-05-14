use kernel::common::registers::{register_bitfields, LocalRegisterCopy};

register_bitfields![u32,
    pub mcause [
        is_interrupt OFFSET(31) NUMBITS(1) [],
        reason OFFSET(0) NUMBITS(31) []
    ],
    // Per the spec, implementations are allowed to use the higher bits of the
    // interrupt/exception reason for their own purposes.  For regular parsing,
    // we only concern ourselves with the "standard" values.
    reason [
        reserved OFFSET(4) NUMBITS(27) [],
        std OFFSET(0) NUMBITS(4) []
    ]
];

/// Trap Cause
#[derive(Copy, Clone, Debug)]
pub enum Trap {
    Interrupt(Interrupt),
    Exception(Exception),
}

impl From<LocalRegisterCopy<u32, mcause::Register>> for Trap {
    fn from(val: LocalRegisterCopy<u32, mcause::Register>) -> Self {
        if val.is_set(mcause::is_interrupt) {
            Trap::Interrupt(Interrupt::from_reason(val.read(mcause::reason)))
        } else {
            Trap::Exception(Exception::from_reason(val.read(mcause::reason)))
        }
    }
}

impl From<u32> for Trap {
    fn from(csr_val: u32) -> Self {
        Self::from(LocalRegisterCopy::<u32, mcause::Register>::new(csr_val))
    }
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
    fn from_reason(val: u32) -> Self {
        let reason = LocalRegisterCopy::<u32, reason::Register>::new(val);
        match reason.read(reason::std) {
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
    fn from_reason(val: u32) -> Self {
        let reason = LocalRegisterCopy::<u32, reason::Register>::new(val);
        match reason.read(reason::std) {
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
