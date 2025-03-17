// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use kernel::utilities::registers::{register_bitfields, LocalRegisterCopy};

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
            Trap::Interrupt(Interrupt::from_reason(val.read(mcause::reason)))
        } else {
            Trap::Exception(Exception::from_reason(val.read(mcause::reason)))
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
            0 => Interrupt::UserSoft,
            1 => Interrupt::SupervisorSoft,
            3 => Interrupt::MachineSoft,
            4 => Interrupt::UserTimer,
            5 => Interrupt::SupervisorTimer,
            7 => Interrupt::MachineTimer,
            8 => Interrupt::UserExternal,
            9 => Interrupt::SupervisorExternal,
            11 => Interrupt::MachineExternal,
            val => Interrupt::Unknown(val),
        }
    }
}

impl Exception {
    fn from_reason(val: usize) -> Self {
        let mcause = LocalRegisterCopy::<usize, mcause::Register>::new(val);
        match mcause.read(mcause::reason) {
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
