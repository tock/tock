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
        if val.is_set(mcause::is_interrupt) {
            Self::Interrupt(Interrupt::from_reason(val.read(mcause::reason)))
        } else {
            Self::Exception(Exception::from_reason(val.read(mcause::reason)))
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

impl Interrupt {
    fn from_reason(val: usize) -> Self {
        let mcause = LocalRegisterCopy::<usize, mcause::Register>::new(val);
        match mcause.read(mcause::reason) {
            0 => Self::UserSoft,
            1 => Self::SupervisorSoft,
            3 => Self::MachineSoft,
            4 => Self::UserTimer,
            5 => Self::SupervisorTimer,
            7 => Self::MachineTimer,
            8 => Self::UserExternal,
            9 => Self::SupervisorExternal,
            11 => Self::MachineExternal,
            val => Self::Unknown(val),
        }
    }
}

impl Exception {
    fn from_reason(val: usize) -> Self {
        let mcause = LocalRegisterCopy::<usize, mcause::Register>::new(val);
        match mcause.read(mcause::reason) {
            0 => Self::InstructionMisaligned,
            1 => Self::InstructionFault,
            2 => Self::IllegalInstruction,
            3 => Self::Breakpoint,
            4 => Self::LoadMisaligned,
            5 => Self::LoadFault,
            6 => Self::StoreMisaligned,
            7 => Self::StoreFault,
            8 => Self::UserEnvCall,
            9 => Self::SupervisorEnvCall,
            11 => Self::MachineEnvCall,
            12 => Self::InstructionPageFault,
            13 => Self::LoadPageFault,
            15 => Self::StorePageFault,
            _ => Self::Unknown,
        }
    }
}
