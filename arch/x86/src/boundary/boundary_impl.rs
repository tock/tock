// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::fmt::Write;

use crate::registers::bits32::eflags::{EFlags, EFLAGS};

use kernel::process::FunctionCall;
use kernel::syscall::{ContextSwitchReason, Syscall, SyscallReturn, UserspaceKernelBoundary};
use kernel::ErrorCode;

use crate::interrupts::{IDT_RESERVED_EXCEPTIONS, SYSCALL_VECTOR};
use crate::segmentation::{USER_CODE, USER_DATA};

use super::UserContext;

/// Defines the usermode-kernelmode ABI for x86 platforms.
pub struct Boundary;

impl Default for Boundary {
    fn default() -> Self {
        Self::new()
    }
}

impl Boundary {
    /// Minimum required size for initial process memory.
    ///
    /// Need at least 9 dwords of initial stack space for CRT 0:
    ///
    /// - 4 dwords for initial upcall arguments
    /// - 1 dword for initial upcall return address (although this will be zero for init_fn)
    /// - 4 dwords of scratch space for invoking memop syscalls
    const MIN_APP_BRK: u32 = 9 * core::mem::size_of::<usize>() as u32;

    /// Constructs a new instance of `SysCall`.
    pub fn new() -> Self {
        Self
    }
}

impl UserspaceKernelBoundary for Boundary {
    type StoredState = UserContext;

    fn initial_process_app_brk_size(&self) -> usize {
        Self::MIN_APP_BRK as usize
    }

    unsafe fn initialize_process(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
    ) -> Result<(), ()> {
        if (app_brk as u32 - accessible_memory_start as u32) < Self::MIN_APP_BRK {
            return Err(());
        }

        // We pre-allocate 16 bytes on the stack for initial upcall arguments.
        let esp = (app_brk as u32) - 16;

        let mut eflags = EFlags::new();
        eflags.0.modify(EFLAGS::FLAGS_IF::SET);

        state.eax = 0;
        state.ebx = 0;
        state.ecx = 0;
        state.edx = 0;
        state.esi = 0;
        state.edi = 0;
        state.ebp = 0;
        state.esp = esp;
        state.eip = 0;
        state.eflags = eflags.0.get();
        state.cs = USER_CODE.bits() as u32;
        state.ss = USER_DATA.bits() as u32;
        state.ds = USER_DATA.bits() as u32;
        state.es = USER_DATA.bits() as u32;
        state.fs = USER_DATA.bits() as u32;
        state.gs = USER_DATA.bits() as u32;

        Ok(())
    }

    unsafe fn set_syscall_return_value(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
        return_value: SyscallReturn,
    ) -> Result<(), ()> {
        let mut ret0 = 0;
        let mut ret1 = 0;
        let mut ret2 = 0;
        let mut ret3 = 0;

        // These operations are only safe so long as
        // - the pointers are properly aligned. This is guaranteed because the
        //   pointers are all offset multiples of 4 bytes from the stack
        //   pointer, which is guaranteed to be properly aligned after
        //   exception entry on x86. See
        //   https://github.com/tock/tock/pull/2478#issuecomment-796389747
        //   for more details.
        // - the pointer is dereferencable, i.e. the memory range of
        //   the given size starting at the pointer must all be within
        //   the bounds of a single allocated object
        // - the pointer must point to an initialized instance of its
        //   type
        // - during the lifetime of the returned reference (of the
        //   cast, essentially an arbitrary 'a), the memory must not
        //   get accessed (read or written) through any other pointer.
        //
        // Refer to
        // https://doc.rust-lang.org/std/primitive.pointer.html#safety-13
        kernel::utilities::arch_helpers::encode_syscall_return_trd104(
            &kernel::utilities::arch_helpers::TRD104SyscallReturn::from_syscall_return(
                return_value,
            ),
            &mut ret0,
            &mut ret1,
            &mut ret2,
            &mut ret3,
        );

        // App allocates 16 bytes of stack space for passing syscall arguments. We re-use that stack
        // space to pass return values.
        //
        // Safety: Caller of this function has guaranteed that the memory region is valid.
        unsafe {
            state.write_stack(0, ret0, accessible_memory_start, app_brk)?;
            state.write_stack(1, ret1, accessible_memory_start, app_brk)?;
            state.write_stack(2, ret2, accessible_memory_start, app_brk)?;
            state.write_stack(3, ret3, accessible_memory_start, app_brk)?;
        }

        Ok(())
    }

    unsafe fn set_process_function(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
        upcall: FunctionCall,
    ) -> Result<(), ()> {
        // Our x86 port expects upcalls to be standard cdecl routines. We push args and return
        // address onto the stack accordingly.
        //
        // Upcall arguments are written directly into the existing stack space (rather than
        // being pushed on top). This is safe to do because:
        //
        // * When the process first starts ESP is initialized to `app_brk - 16`, giving us exactly
        //   enough space for these arguments.
        // * Otherwise, we assume the app is currently issuing a `yield` syscall. We re-use the
        //   stack space from that syscall. This is okay because `yield` doesn't return anything.
        //
        // Safety: Caller of this function has guaranteed that the memory region is valid.
        // usize is u32 on x86
        unsafe {
            state.write_stack(0, upcall.argument0 as u32, accessible_memory_start, app_brk)?;
            state.write_stack(1, upcall.argument1 as u32, accessible_memory_start, app_brk)?;
            state.write_stack(2, upcall.argument2 as u32, accessible_memory_start, app_brk)?;
            state.write_stack(
                3,
                upcall.argument3.as_usize() as u32,
                accessible_memory_start,
                app_brk,
            )?;

            state.push_stack(state.eip, accessible_memory_start, app_brk)?;
        }

        // The next time we switch to this process, we will directly jump to the upcall. When the
        // upcall issues `ret`, it will return to wherever the yield syscall was invoked.
        state.eip = upcall.pc.addr() as u32;

        Ok(())
    }

    unsafe fn switch_to_process(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
    ) -> (ContextSwitchReason, Option<*const u8>) {
        // Sanity check: don't try to run a faulted app
        if state.exception != 0 || state.err_code != 0 {
            let stack_ptr = state.esp as *mut u8;
            return (ContextSwitchReason::Fault, Some(stack_ptr));
        }

        let mut err_code = 0;
        let int_num = unsafe { super::switch_to_user(state, &mut err_code) };

        let reason = match int_num as u8 {
            0..IDT_RESERVED_EXCEPTIONS => {
                state.exception = int_num as u8;
                state.err_code = err_code;
                ContextSwitchReason::Fault
            }

            SYSCALL_VECTOR => {
                let num = state.eax as u8;

                // Syscall arguments are passed on the stack using cdecl convention.
                //
                // Safety: Caller of this function has guaranteed that the memory region is valid.
                let arg0 =
                    unsafe { state.read_stack(0, accessible_memory_start, app_brk) }.unwrap_or(0);
                let arg1 =
                    unsafe { state.read_stack(1, accessible_memory_start, app_brk) }.unwrap_or(0);
                let arg2 =
                    unsafe { state.read_stack(2, accessible_memory_start, app_brk) }.unwrap_or(0);
                let arg3 =
                    unsafe { state.read_stack(3, accessible_memory_start, app_brk) }.unwrap_or(0);

                Syscall::from_register_arguments(
                    num,
                    arg0 as usize,
                    (arg1 as usize).into(),
                    (arg2 as usize).into(),
                    (arg3 as usize).into(),
                )
                .map_or(ContextSwitchReason::Fault, |syscall| {
                    ContextSwitchReason::SyscallFired { syscall }
                })
            }
            _ => ContextSwitchReason::Interrupted,
        };

        let stack_ptr = state.esp as *const u8;

        (reason, Some(stack_ptr))
    }

    unsafe fn print_context(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        state: &Self::StoredState,
        writer: &mut dyn Write,
    ) {
        let _ = writeln!(writer, "{}", state);
    }

    fn store_context(
        &self,
        _state: &Self::StoredState,
        _out: &mut [u8],
    ) -> Result<usize, ErrorCode> {
        unimplemented!()
    }
}
