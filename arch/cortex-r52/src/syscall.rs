// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! System call (SVC) handling for Cortex-R52.
//!
use core::fmt::Write;
use core::ptr::{self, addr_of, addr_of_mut, read_volatile, write_volatile};
use kernel::errorcode::ErrorCode;

/// Set by the SVC handler when userspace executes an `svc` instruction.
#[unsafe(no_mangle)]
#[used]
pub static mut SYSCALL_FIRED: usize = 0;

/// Process state preserved across kernel entries.
#[derive(Default)]
pub struct CortexR52StoredState {
    // What else !?
    sp_usr: usize,
}

// Space for 8 u32s: r0-r12, ret_addr, cpsr, lr
// Check switch_to_user_arm_cr52() for frame description
const SVC_FRAME_SIZE: usize = 16 * 4; // 16 words, 4 bytes each

// From the doc: SPSR (MSB-LSB-meaning)
//    31-27  condition flags = 0
//    26-25  If Then flags(1) = 0
//    24     J = 0
//    23     SBSS ? = 0
//    22     PAN ? = 0
//    21     DIT ? = 0
//    20     Illegal Execution State = 0
//    19-16  GT or EQ flags = 0
//    15-10  If Then flags(2) = 0
//     9     Endianness = 0
//     8     A / Sys error mask = 0
//     7     I / IRQ mask = 0
//     6     F / FIQ mask = 0
//     5     T / T32 state = 1
//     4--0  M / mode = 0b10000 (USR)
const SPSR_USR_DEF: usize = 0b0000_0000_0001_0000; //  MODE_USR

impl core::convert::TryFrom<&[u8]> for CortexR52StoredState {
    type Error = ErrorCode;

    // Do not know why this is needed
    fn try_from(_ss: &[u8]) -> Result<CortexR52StoredState, Self::Error> {
        todo!()
    }
}

/// Implementation of the [`UserspaceKernelBoundary`](kernel::syscall::UserspaceKernelBoundary)
/// for Cortex-R52.
pub struct SysCall;

impl SysCall {
    pub const unsafe fn new() -> SysCall {
        SysCall
    }
}

impl kernel::syscall::UserspaceKernelBoundary for SysCall {
    type StoredState = CortexR52StoredState;

    fn initial_process_app_brk_size(&self) -> usize {
        SVC_FRAME_SIZE
    }

    unsafe fn initialize_process(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
    ) -> Result<(), ()> {
        state.sp_usr = app_brk as usize;

        // just copied from Cortex-M (didn't check)
        if (app_brk as usize - accessible_memory_start as usize) < SVC_FRAME_SIZE {
            return Err(());
        }

        state.sp_usr -= SVC_FRAME_SIZE;

        // Color with 0xBB for debugging purposes
        ptr::write_bytes(state.sp_usr as *mut usize, 0xBB, SVC_FRAME_SIZE);

        Ok(())
    }

    unsafe fn set_syscall_return_value(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
        return_value: kernel::syscall::SyscallReturn,
    ) -> Result<(), ()> {
        // just copied from Cortex-M (didn't check)
        if state.sp_usr < accessible_memory_start as usize
            || state.sp_usr.saturating_add(core::mem::size_of::<u32>() * 4) > app_brk as usize
        {
            return Err(());
        }

        // Get register values from the right place in the user stack frame
        let sp = state.sp_usr as *mut u32;
        let (r0, r1, r2, r3) = (sp.offset(3), sp.offset(4), sp.offset(5), sp.offset(6));

        //sh::p!("--- syscall return r0:{:x} r1:{:x} r2:{:x} r3:{:x}", *r0, *r1, *r2, *r3);

        kernel::utilities::arch_helpers::encode_syscall_return_trd104(
            //&kernel::utilities::arch_helpers::TRD104SyscallReturn::from_syscall_return(return_value,),
            &return_value.into(),
            &mut *r0,
            &mut *r1,
            &mut *r2,
            &mut *r3,
        );

        Ok(())
    }

    unsafe fn set_process_function(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
        callback: kernel::process::FunctionCall,
    ) -> Result<(), ()> {
        //sh::p!("---set_process_function {callback:?}");

        // just copied from Cortex-M (didn't check)
        if state.sp_usr < accessible_memory_start as usize
            || state.sp_usr.saturating_add(SVC_FRAME_SIZE) > app_brk as usize
        {
            return Err(());
        }

        // Set register values to the right place in the user stack frame
        let stack_bottom = state.sp_usr as *mut usize;
        ptr::write(stack_bottom.offset(0), *stack_bottom.offset(2)); // Set LR with previous PC
        ptr::write(stack_bottom.offset(1), SPSR_USR_DEF); // Default PSTATE
        ptr::write(stack_bottom.offset(2), callback.pc.addr()); // Set PC withc callback
        ptr::write(stack_bottom.offset(3), callback.argument0); // R0...
        ptr::write(stack_bottom.offset(4), callback.argument1);
        ptr::write(stack_bottom.offset(5), callback.argument2);
        ptr::write(stack_bottom.offset(6), callback.argument3.as_usize()); // ...R3

        Ok(())
    }

    unsafe fn switch_to_process(
        &self,
        accessible_memory_start: *const u8,
        app_brk: *const u8,
        state: &mut Self::StoredState,
    ) -> (kernel::syscall::ContextSwitchReason, Option<*const u8>) {
        //sh::p!("--- switch_to_process BEG");

        // Big stuff occuring here: call the asm context switch and run user code
        let new_stack_pointer =
            crate::vectors::switch_to_user_arm_cr52(state.sp_usr as *const usize);
        // when we return via SVC or FIQ handlers, we are back in the kernel context

        // Save the user stack pointer returned by the handler.
        state.sp_usr = new_stack_pointer as usize;

        // just copied from Cortex-M
        let invalid_stack_pointer = state.sp_usr < accessible_memory_start as usize
            || state.sp_usr.saturating_add(SVC_FRAME_SIZE) > app_brk as usize;

        if invalid_stack_pointer {
            return (kernel::syscall::ContextSwitchReason::Fault, None);
        }

        // Consume the SVC marker set by the assembly handler.
        let syscall_fired = read_volatile(&*addr_of!(SYSCALL_FIRED));
        write_volatile(&mut *addr_of_mut!(SYSCALL_FIRED), 0);

        let switch_reason = if syscall_fired == 1 {
            // Get the SVC arguments from user stack frame
            let r0 = ptr::read(new_stack_pointer.offset(3));
            let r1 = ptr::read(new_stack_pointer.offset(4));
            let r2 = ptr::read(new_stack_pointer.offset(5));
            let r3 = ptr::read(new_stack_pointer.offset(6));

            // Get the SVC num from the very instruction
            let lr = ptr::read(new_stack_pointer.offset(2)) as *const u32;
            let svc_instr = ptr::read(lr.offset(-1));
            let svc_num = (svc_instr & 0xff) as u8;

            // debug
            //let a = ptr::read(new_stack_pointer.offset(1));
            //let b = ptr::read(new_stack_pointer.offset(2));
            //sh::p!("--- switch_to_process SC LR:{lr:?} 1:{a:x} 2:{b:x} R0:{r0:x} R1:{r1:x} R2:{r2:x} R3:{r3:x} SVC{svc_num}");

            let syscall = kernel::utilities::arch_helpers::syscall_from_register_arguments_trd104(
                svc_num,
                r0,
                r1.into(),
                r2.into(),
                r3.into(),
            );

            match syscall {
                Some(s) => kernel::syscall::ContextSwitchReason::SyscallFired { syscall: s },
                None => kernel::syscall::ContextSwitchReason::Fault,
            }
        } else {
            //sh::p!("--- switch_to_process IRQ");
            kernel::syscall::ContextSwitchReason::Interrupted
        };

        (switch_reason, Some(state.sp_usr as *const u8))
    }

    unsafe fn print_context(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        _state: &Self::StoredState,
        _writer: &mut dyn Write,
    ) {
        todo!();
        /*
        // Copied from Cortex M, need rework
        let invalid_stack_pointer = state.psp < accessible_memory_start as usize
            || state.psp.saturating_add(SVC_FRAME_SIZE) > app_brk as usize;

        let stack_pointer = state.psp as *const usize;

        let (r0, r1, r2, r3, r12, lr, pc, xpsr) = if invalid_stack_pointer {
            (
                0xBAD00BAD, 0xBAD00BAD, 0xBAD00BAD, 0xBAD00BAD, 0xBAD00BAD, 0xBAD00BAD, 0xBAD00BAD,
                0xBAD00BAD,
            )
        } else {
            let r0 = ptr::read(stack_pointer.offset(0));
            let r1 = ptr::read(stack_pointer.offset(1));
            let r2 = ptr::read(stack_pointer.offset(2));
            let r3 = ptr::read(stack_pointer.offset(3));
            let r12 = ptr::read(stack_pointer.offset(4));
            let lr = ptr::read(stack_pointer.offset(5));
            let pc = ptr::read(stack_pointer.offset(6));
            let xpsr = ptr::read(stack_pointer.offset(7));
            (r0, r1, r2, r3, r12, lr, pc, xpsr)
        };

        let _ = writer.write_fmt(format_args!(
            "\
             \r\n  R0 : {:#010X}    R6 : {:#010X}\
             \r\n  R1 : {:#010X}    R7 : {:#010X}\
             \r\n  R2 : {:#010X}    R8 : {:#010X}\
             \r\n  R3 : {:#010X}    R10: {:#010X}\
             \r\n  R4 : {:#010X}    R11: {:#010X}\
             \r\n  R5 : {:#010X}    R12: {:#010X}\
             \r\n  R9 : {:#010X} (Static Base Register)\
             \r\n  SP : {:#010X} (Process Stack Pointer)\
             \r\n  LR : {:#010X}\
             \r\n  PC : {:#010X}\
             \r\n YPC : {:#010X}\
             \r\n",
            r0,
            state.regs[2],
            r1,
            state.regs[3],
            r2,
            state.regs[4],
            r3,
            state.regs[6],
            state.regs[0],
            state.regs[7],
            state.regs[1],
            r12,
            state.regs[5],
            stack_pointer as usize,
            lr,
            pc,
            state.yield_pc,
        ));

        let _ = writer.write_fmt(format_args!(
            "\
             \r\n APSR: N {} Z {} C {} V {} Q {}\
             \r\n       GE {} {} {} {}",
            (xpsr >> 31) & 0x1,
            (xpsr >> 30) & 0x1,
            (xpsr >> 29) & 0x1,
            (xpsr >> 28) & 0x1,
            (xpsr >> 27) & 0x1,
            (xpsr >> 19) & 0x1,
            (xpsr >> 18) & 0x1,
            (xpsr >> 17) & 0x1,
            (xpsr >> 16) & 0x1,
        ));

        let ici_it = (((xpsr >> 25) & 0x3) << 6) | ((xpsr >> 10) & 0x3f);
        let thumb_bit = ((xpsr >> 24) & 0x1) == 1;
        let _ = writer.write_fmt(format_args!(
            "\
             \r\n EPSR: ICI.IT {:#04x}\
             \r\n       ThumbBit {} {}\r\n",
            ici_it,
            thumb_bit,
            if thumb_bit {
                ""
            } else {
                "!!ERROR - Cortex R52 Thumb only!"
            },
        ));
        */
    }

    // Do not know why this is needed
    fn store_context(
        &self,
        _state: &Self::StoredState,
        _out: &mut [u8],
    ) -> Result<usize, ErrorCode> {
        Err(ErrorCode::SIZE)
    }
}
