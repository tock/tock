// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::fmt::{self, Display, Formatter};
use core::mem::size_of;
use core::ptr;

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

impl UserContext {
    /// Pushes a value onto the user stack.
    ///
    /// Returns an `Err` if the new stack value would fall outside of valid memory.
    ///
    /// ## Safety
    ///
    /// The memory region described by `accessible_memory_start` and `app_brk` must point to memory
    /// of the user process. This function will write to that memory.
    pub unsafe fn push_stack(
        &mut self,
        value: u32,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
    ) -> Result<(), ()> {
        let new_esp = self.esp - 4;

        if new_esp < accessible_memory_start as u32 {
            return Err(());
        }

        if new_esp + 4 > app_brk as u32 {
            return Err(());
        }

        // Safety: We have validated above that new_esp lies within the specified memory region, and
        //         the caller has guaranteed that this region is valid.
        unsafe { ptr::write_volatile(new_esp as *mut u32, value) };

        self.esp = new_esp;

        Ok(())
    }

    /// Reads a value from `offset` relative to the current user stack pointer.
    ///
    /// `offset` is a DWORD offset (i.e. 4 bytes), not bytes.
    ///
    /// Returns an `Err` if the specified location falls outside of valid memory.
    ///
    /// ## Safety
    ///
    /// The memory region described by `accessible_memory_start` and `app_brk` must point to memory
    /// of the user process. This function will read from that memory.
    pub unsafe fn read_stack(
        &self,
        offset: u32,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
    ) -> Result<u32, ()> {
        let stack_addr = self.esp + (offset * 4);

        if stack_addr < accessible_memory_start as u32 {
            return Err(());
        }

        if stack_addr + 4 > app_brk as u32 {
            return Err(());
        }

        // Safety: We have validated above that stack_addr lies within the specified memory region,
        //         and the caller has guaranteed that this region is valid.
        let val = unsafe { ptr::read_volatile(stack_addr as *mut u32) };

        Ok(val)
    }

    /// Writes a value to `offset` relative to the current user stack pointer.
    ///
    /// `offset` is a DWORD offset (i.e. 4 bytes), not bytes.
    ///
    /// Returns an `Err` if the specified location falls outside of valid memory.
    ///
    /// ## Safety
    ///
    /// The memory region described by `accessible_memory_start` and `app_brk` must point to memory
    /// of the user process. This function will write to that memory.
    pub unsafe fn write_stack(
        &self,
        offset: u32,
        value: u32,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
    ) -> Result<(), ()> {
        let stack_addr = self.esp + (offset * size_of::<usize>() as u32);

        if stack_addr < accessible_memory_start as u32 {
            return Err(());
        }

        if stack_addr + 4 > app_brk as u32 {
            return Err(());
        }

        // Safety: We have validated above that stack_addr lies within the specified memory region,
        //         and the caller has guaranteed that this region is valid.
        unsafe { ptr::write_volatile(stack_addr as *mut u32, value) };

        Ok(())
    }
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
