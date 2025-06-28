// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::fmt::{self, Display, Formatter};

use crate::registers::irq::EXCEPTIONS;

/// Stored CPU state of a user-mode app
///
/// This struct stores the complete CPU state of a user-mode Tock application on x86.
///
/// We access this struct from several assembly routines to perform context switching between user
/// and kernel mode. For this reason, it is **critical** that the struct have a deterministic layout
/// in memory. We use `#[repr(C)]` for this.
#[repr(C)]
#[derive(Default)]
pub struct UserContext {
    pub eax: u32,    // Offset:  0
    pub ebx: u32,    // Offset:  4
    pub ecx: u32,    // Offset:  8
    pub edx: u32,    // Offset: 12
    pub esi: u32,    // Offset: 16
    pub edi: u32,    // Offset: 20
    pub ebp: u32,    // Offset: 24
    pub esp: u32,    // Offset: 28
    pub eip: u32,    // Offset: 32
    pub eflags: u32, // Offset: 36
    pub cs: u32,     // Offset: 40
    pub ss: u32,     // Offset: 44
    pub ds: u32,     // Offset: 48
    pub es: u32,     // Offset: 52
    pub fs: u32,     // Offset: 56
    pub gs: u32,     // Offset: 60

    /// If the process triggers a CPU exception, this field will be populated with the
    /// exception number. Otherwise this field must remain as zero.
    pub exception: u8,

    /// If the process triggers a CPU exception with an associated error code, this field will be
    /// populated with the error code value. Otherwise this field must remain zero.
    pub err_code: u32,
}

impl Display for UserContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, " CPU Registers:")?;
        writeln!(f)?;
        writeln!(f, "  EAX: {:#010x}      EBX: {:#010x}", self.eax, self.ebx)?;
        writeln!(f, "  ECX: {:#010x}      EDX: {:#010x}", self.ecx, self.edx)?;
        writeln!(f, "  ESI: {:#010x}      EDI: {:#010x}", self.esi, self.edi)?;
        writeln!(f, "  EBP: {:#010x}      ESP: {:#010x}", self.ebp, self.esp)?;
        writeln!(
            f,
            "  EIP: {:#010x}   EFLAGS: {:#010x}",
            self.eip, self.eflags
        )?;
        writeln!(
            f,
            "   CS:     {:#06x}       SS:     {:#06x}",
            self.cs, self.ss
        )?;
        writeln!(
            f,
            "   DS:     {:#06x}       ES:     {:#06x}",
            self.ds, self.es
        )?;
        writeln!(
            f,
            "   FS:     {:#06x}       GS:     {:#06x}",
            self.fs, self.gs
        )?;

        if self.exception != 0 || self.err_code != 0 {
            writeln!(f)?;
            if let Some(msg) = EXCEPTIONS.get(self.exception as usize) {
                writeln!(f, " Exception: {}", msg)?;
            } else {
                writeln!(f, " Exception Number: {}", self.exception)?;
            }
            writeln!(f, " Error code: {:#010x}", self.err_code)?;
        }
        Ok(())
    }
}
