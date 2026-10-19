// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::registers::{LocalRegisterCopy, register_bitfields};

register_bitfields![usize,
    pub mcause [
        is_interrupt OFFSET(crate::XLEN - 1) NUMBITS(1) [],
        reason OFFSET(0) NUMBITS(crate::XLEN - 1) []
    ],
];

/// Trap Cause
#[derive(Copy, Clone, Debug)]
pub enum Trap {
    Interrupt(Interrupt),
    Exception(Exception),
}

impl From<LocalRegisterCopy<usize, mcause::Register>> for Trap {
    fn from(val: LocalRegisterCopy<usize, mcause::Register>) -> Self {
        let code = val.read(mcause::reason);
        if val.is_set(mcause::is_interrupt) {
            Trap::Interrupt(Interrupt::try_from(code).unwrap_or(Interrupt::Unknown(code)))
        } else {
            Trap::Exception(Exception::try_from(code).unwrap_or(Exception::Unknown))
        }
    }
}

impl From<usize> for Trap {
    fn from(csr_val: usize) -> Self {
        Self::from(LocalRegisterCopy::<usize, mcause::Register>::new(csr_val))
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
    Unknown(usize),
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

impl TryFrom<usize> for Interrupt {
    type Error = usize;

    fn try_from(code: usize) -> Result<Self, Self::Error> {
        Ok(match code {
            0 => Interrupt::UserSoft,
            1 => Interrupt::SupervisorSoft,
            3 => Interrupt::MachineSoft,
            4 => Interrupt::UserTimer,
            5 => Interrupt::SupervisorTimer,
            7 => Interrupt::MachineTimer,
            8 => Interrupt::UserExternal,
            9 => Interrupt::SupervisorExternal,
            11 => Interrupt::MachineExternal,
            _ => return Err(code),
        })
    }
}

impl TryFrom<usize> for Exception {
    type Error = usize;

    fn try_from(code: usize) -> Result<Self, Self::Error> {
        Ok(match code {
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
            _ => return Err(code),
        })
    }
}
