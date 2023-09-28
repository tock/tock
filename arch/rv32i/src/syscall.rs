// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Kernel-userland system call interface for RISC-V architecture.

use core::convert::TryInto;
use core::fmt::Write;
use core::mem::size_of;
use core::ops::Range;

use crate::csr::mcause;
use kernel;
use kernel::errorcode::ErrorCode;
use kernel::syscall::ContextSwitchReason;

/// This holds all of the state that the kernel must keep for the process when
/// the process is not executing.
#[derive(Default)]
#[repr(C)]
pub struct Riscv32iStoredState {
    /// Store all of the app registers.
    regs: [u32; 31],

    /// This holds the PC value of the app when the exception/syscall/interrupt
    /// occurred. We also use this to set the PC that the app should start
    /// executing at when it is resumed/started.
    pc: u32,

    /// We need to store the mcause CSR between when the trap occurs and after
    /// we exit the trap handler and resume the context switching code.
    mcause: u32,

    /// We need to store the mtval CSR for the process in case the mcause
    /// indicates a fault. In that case, the mtval contains useful debugging
    /// information.
    mtval: u32,
}

// Named offsets into the stored state registers.  These needs to be kept in
// sync with the register save logic in _start_trap() as well as the register
// restore logic in switch_to_process() below.
const R_RA: usize = 0;
const R_SP: usize = 1;
const R_A0: usize = 9;
const R_A1: usize = 10;
const R_A2: usize = 11;
const R_A3: usize = 12;
const R_A4: usize = 13;

/// Values for encoding the stored state buffer in a binary slice.
const VERSION: u32 = 1;
const STORED_STATE_SIZE: u32 = size_of::<Riscv32iStoredState>() as u32;
const TAG: [u8; 4] = [b'r', b'v', b'5', b'i'];
const METADATA_LEN: usize = 3;

const VERSION_IDX: usize = 0;
const SIZE_IDX: usize = 1;
const TAG_IDX: usize = 2;
const PC_IDX: usize = 3;
const MCAUSE_IDX: usize = 4;
const MTVAL_IDX: usize = 5;
const REGS_IDX: usize = 6;
const REGS_RANGE: Range<usize> = REGS_IDX..REGS_IDX + 31;

const U32_SZ: usize = size_of::<u32>();
fn u32_byte_range(index: usize) -> Range<usize> {
    index * U32_SZ..(index + 1) * U32_SZ
}

fn u32_from_u8_slice(slice: &[u8], index: usize) -> Result<u32, ErrorCode> {
    let range = u32_byte_range(index);
    Ok(u32::from_le_bytes(
        slice
            .get(range)
            .ok_or(ErrorCode::SIZE)?
            .try_into()
            .or(Err(ErrorCode::FAIL))?,
    ))
}

fn write_u32_to_u8_slice(val: u32, slice: &mut [u8], index: usize) {
    let range = u32_byte_range(index);
    slice[range].copy_from_slice(&val.to_le_bytes());
}

impl core::convert::TryFrom<&[u8]> for Riscv32iStoredState {
    type Error = ErrorCode;
    fn try_from(ss: &[u8]) -> Result<Riscv32iStoredState, Self::Error> {
        if ss.len() == size_of::<Riscv32iStoredState>() + METADATA_LEN * U32_SZ
            && u32_from_u8_slice(ss, VERSION_IDX)? == VERSION
            && u32_from_u8_slice(ss, SIZE_IDX)? == STORED_STATE_SIZE
            && u32_from_u8_slice(ss, TAG_IDX)? == u32::from_le_bytes(TAG)
        {
            let mut res = Riscv32iStoredState {
                regs: [0; 31],
                pc: u32_from_u8_slice(ss, PC_IDX)?,
                mcause: u32_from_u8_slice(ss, MCAUSE_IDX)?,
                mtval: u32_from_u8_slice(ss, MTVAL_IDX)?,
            };
            for (i, v) in (REGS_RANGE).enumerate() {
                res.regs[i] = u32_from_u8_slice(ss, v)?;
            }
            Ok(res)
        } else {
            Err(ErrorCode::FAIL)
        }
    }
}

/// Implementation of the `UserspaceKernelBoundary` for the RISC-V architecture.
pub struct SysCall(());

impl SysCall {
    pub const unsafe fn new() -> SysCall {
        SysCall(())
    }
}

impl kernel::syscall::UserspaceKernelBoundary for SysCall {
    type StoredState = Riscv32iStoredState;

    fn initial_process_app_brk_size(&self) -> usize {
        // The RV32I UKB implementation does not use process memory for any
        // context switch state. Therefore, we do not need any process-accessible
        // memory to start with to successfully context switch to the process the
        // first time.
        0
    }

    unsafe fn initialize_process(
        &self,
        accessible_memory_start: *const u8,
        _app_brk: *const u8,
        state: &mut Self::StoredState,
    ) -> Result<(), ()> {
        // Need to clear the stored state when initializing.
        state.regs.iter_mut().for_each(|x| *x = 0);
        state.pc = 0;
        state.mcause = 0;

        // The first time the process runs we need to set the initial stack
        // pointer in the sp register.
        //
        // We do not pre-allocate any stack for RV32I processes.
        state.regs[R_SP] = accessible_memory_start as u32;

        // We do not use memory for UKB, so just return ok.
        Ok(())
    }

    unsafe fn set_syscall_return_value(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        state: &mut Self::StoredState,
        return_value: kernel::syscall::SyscallReturn,
    ) -> Result<(), ()> {
        // Encode the system call return value into registers,
        // available for when the process resumes

        // We need to use a bunch of split_at_mut's to have multiple
        // mutable borrows into the same slice at the same time.
        //
        // Since the compiler knows the size of this slice, and these
        // calls will be optimized out, we use one to get to the first
        // register (A0)
        let (_, r) = state.regs.split_at_mut(R_A0);

        // This comes with the assumption that the respective
        // registers are stored at monotonically increasing indices
        // in the register slice
        let (a0slice, r) = r.split_at_mut(R_A1 - R_A0);
        let (a1slice, r) = r.split_at_mut(R_A2 - R_A1);
        let (a2slice, a3slice) = r.split_at_mut(R_A3 - R_A2);

        return_value.encode_syscall_return(
            &mut a0slice[0],
            &mut a1slice[0],
            &mut a2slice[0],
            &mut a3slice[0],
        );

        // We do not use process memory, so this cannot fail.
        Ok(())
    }

    unsafe fn set_process_function(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        state: &mut Riscv32iStoredState,
        callback: kernel::process::FunctionCall,
    ) -> Result<(), ()> {
        // Set the register state for the application when it starts
        // executing. These are the argument registers.
        state.regs[R_A0] = callback.argument0 as u32;
        state.regs[R_A1] = callback.argument1 as u32;
        state.regs[R_A2] = callback.argument2 as u32;
        state.regs[R_A3] = callback.argument3 as u32;

        // We also need to set the return address (ra) register so that the new
        // function that the process is running returns to the correct location.
        // Note, however, that if this function happens to be the first time the
        // process is executing then `state.pc` is invalid/useless, but the
        // application must ignore it anyway since there is nothing logically
        // for it to return to. So this doesn't hurt anything.
        state.regs[R_RA] = state.pc;

        // Save the PC we expect to execute.
        state.pc = callback.pc as u32;

        Ok(())
    }

    // Mock implementation for tests on Travis-CI.
    #[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
    unsafe fn switch_to_process(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        _state: &mut Riscv32iStoredState,
    ) -> (ContextSwitchReason, Option<*const u8>) {
        // Convince lint that 'mcause' and 'R_A4' are used during test build
        let _cause = mcause::Trap::from(_state.mcause as usize);
        let _arg4 = _state.regs[R_A4];
        unimplemented!()
    }

    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    unsafe fn switch_to_process(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        state: &mut Riscv32iStoredState,
    ) -> (ContextSwitchReason, Option<*const u8>) {
        use core::arch::asm;
        // We need to ensure that the compiler does not reorder
        // kernel memory writes to after the userspace context switch
        // to ensure we provide a consistent memory view of
        // application-accessible buffers.
        //
        // The compiler will not be able to reorder memory accesses
        // beyond this point, as the "nomem" option on the asm!-block
        // is not set, hence the compiler has to assume the assembly
        // will issue arbitrary memory accesses (acting as a compiler
        // fence).
        asm!("
          // Before switching to the app we need to save some kernel registers
          // to the kernel stack, specifically ones which we can't mark as
          // clobbered in the asm!() block. We then save the stack pointer in
          // the mscratch CSR (0x340) so we can retrieve it after returning to
          // the kernel from the app.
          //
          // A few values get saved to the kernel stack, including an app
          // register temporarily after entering the trap handler. Here is a
          // memory map to make it easier to keep track:
          //
          // ```
          //  8*4(sp):          <- original stack pointer
          //  7*4(sp):
          //  6*4(sp): x9
          //  5*4(sp): x8
          //  4*4(sp): x4
          //  3*4(sp): x3
          //  2*4(sp): _return_to_kernel (100) (address to resume after trap)
          //  1*4(sp): *state   (Per-process StoredState struct)
          //  0*4(sp): app s0   <- new stack pointer
          // ```

          addi sp, sp, -8*4  // Move the stack pointer down to make room.

          // Save all registers on the kernel stack which cannot be clobbered
          // by an asm!() block. These are mostly registers which have a
          // designated purpose (e.g. stack pointer) or are used internally
          // by LLVM.
          //   x2             // sp -> saved in mscratch CSR below
          sw   x3,  3*4(sp)   // gp (can't be clobbered / used as an operand)
          sw   x4,  4*4(sp)   // tp (can't be clobbered / used as an operand)
          sw   x8,  5*4(sp)   // fp (can't be clobbered / used as an operand)
          sw   x9,  6*4(sp)   // s1 (used internally by LLVM)

          sw   a0, 1*4(sp)    // Store process state pointer on stack as well.
                              // We need to have this available for after the app
                              // returns to the kernel so we can store its
                              // registers.

          // From here on we can't allow the CPU to take interrupts
          // anymore, as that might result in the trap handler
          // believing that a context switch to userspace already
          // occurred (as mscratch is non-zero). Restore the userspace
          // state fully prior to enabling interrupts again
          // (implicitly using mret).
          //
          // If this is executed _after_ setting mscratch, this result
          // in the race condition of [PR
          // 2308](https://github.com/tock/tock/pull/2308)

          // Therefore, clear the following bits in mstatus first:
          //   0x00000008 -> bit 3 -> MIE (disabling interrupts here)
          // + 0x00001800 -> bits 11,12 -> MPP (switch to usermode on mret)
          li t0, 0x00001808
          csrrc x0, 0x300, t0      // clear bits in mstatus, don't care about read

          // Afterwards, set the following bits in mstatus:
          //   0x00000080 -> bit 7 -> MPIE (enable interrupts on mret)
          li t0, 0x00000080
          csrrs x0, 0x300, t0      // set bits in mstatus, don't care about read


          // Store the address to jump back to on the stack so that the trap
          // handler knows where to return to after the app stops executing.
          //
          // In asm!() we can't use the shorthand `li` pseudo-instruction, as it
          // complains about _return_to_kernel (100) not being a constant in the
          // required range.
          lui  t0, %hi(100f)
          addi t0, t0, %lo(100f)
          sw   t0, 2*4(sp)

          csrw 0x340, sp      // Save stack pointer in mscratch. This allows
                              // us to find it when the app returns back to
                              // the kernel.

          // We have to set the mepc CSR with the PC we want the app to start
          // executing at. This has been saved in Riscv32iStoredState for us
          // (either when the app returned back to the kernel or in the
          // `set_process_function()` function).
          lw   t0, 31*4(a0)   // Retrieve the PC from Riscv32iStoredState
          csrw 0x341, t0      // Set mepc CSR. This is the PC we want to go to.

          // Restore all of the app registers from what we saved. If this is the
          // first time running the app then most of these values are
          // irrelevant, However we do need to set the four arguments to the
          // `_start_ function in the app. If the app has been executing then this
          // allows the app to correctly resume.
          mv   t0,  a0       // Save the state pointer to a specific register.
          lw   x1,  0*4(t0)  // ra
          lw   x2,  1*4(t0)  // sp
          lw   x3,  2*4(t0)  // gp
          lw   x4,  3*4(t0)  // tp
          lw   x6,  5*4(t0)  // t1
          lw   x7,  6*4(t0)  // t2
          lw   x8,  7*4(t0)  // s0,fp
          lw   x9,  8*4(t0)  // s1
          lw   x10, 9*4(t0)  // a0
          lw   x11, 10*4(t0) // a1
          lw   x12, 11*4(t0) // a2
          lw   x13, 12*4(t0) // a3
          lw   x14, 13*4(t0) // a4
          lw   x15, 14*4(t0) // a5
          lw   x16, 15*4(t0) // a6
          lw   x17, 16*4(t0) // a7
          lw   x18, 17*4(t0) // s2
          lw   x19, 18*4(t0) // s3
          lw   x20, 19*4(t0) // s4
          lw   x21, 20*4(t0) // s5
          lw   x22, 21*4(t0) // s6
          lw   x23, 22*4(t0) // s7
          lw   x24, 23*4(t0) // s8
          lw   x25, 24*4(t0) // s9
          lw   x26, 25*4(t0) // s10
          lw   x27, 26*4(t0) // s11
          lw   x28, 27*4(t0) // t3
          lw   x29, 28*4(t0) // t4
          lw   x30, 29*4(t0) // t5
          lw   x31, 30*4(t0) // t6
          lw   x5,  4*4(t0)  // t0. Do last since we overwrite our pointer.

          // Call mret to jump to where mepc points, switch to user mode, and
          // start running the app.
          mret




          // This is where the trap handler jumps back to after the app stops
          // executing.
        100: // _return_to_kernel

          // We have already stored the app registers in the trap handler. We
          // can restore the kernel registers before resuming kernel code.
          //   x2            // sp -> loaded from mscratch by the trap handler
          lw   x3,  3*4(sp)  // gp (can't be clobbered / used as an operand)
          lw   x4,  4*4(sp)  // tp (can't be clobbered / used as an operand)
          lw   x8,  5*4(sp)  // fp (can't be clobbered / used as an operand)
          lw   x9,  6*4(sp)  // s1 (used internally by LLVM)

          lw   a0,  1*4(sp)  // Restore the the process state pointer such that
                             // we don't need to mark it as clobbered.
                             // Otherwise, this would cause Rust to stack a
                             // register which we already manually save.

          addi sp, sp, 8*4   // Reset kernel stack pointer
          ",

          // The register to put the state struct pointer in is not
          // particularly relevant, however we must avoid using t0
          // as that is overwritten prior to being accessed
          // (although stored and later restored) in the assembly
          in("a0") state as *mut Riscv32iStoredState,

          // Clobber all registers which can be marked as clobbered, except
          // for `a0` / `x10`. By making it retain the value of `&mut state`,
          // which we need to stack manually anyway, we can avoid Rust/LLVM
          // stacking it redundantly for us.
          out("x1") _, out("x5") _, out("x6") _, out("x7") _, out("x11") _,
          out("x12") _, out("x13") _, out("x14") _, out("x15") _, out("x16") _,
          out("x17") _, out("x18") _, out("x19") _, out("x20") _, out("x21") _,
          out("x22") _, out("x23") _, out("x24") _, out("x25") _, out("x26") _,
          out("x27") _, out("x28") _, out("x29") _, out("x30") _, out("x31") _,
        );

        let ret = match mcause::Trap::from(state.mcause as usize) {
            mcause::Trap::Interrupt(_intr) => {
                // An interrupt occurred while the app was running.
                ContextSwitchReason::Interrupted
            }
            mcause::Trap::Exception(excp) => {
                match excp {
                    // The SiFive HiFive1 board allegedly does not support
                    // u-mode, so the m-mode ecall is handled here too.
                    mcause::Exception::UserEnvCall | mcause::Exception::MachineEnvCall => {
                        // Need to increment the PC so when we return we start at the correct
                        // instruction. The hardware does not do this for us.
                        state.pc += 4;

                        let syscall = kernel::syscall::Syscall::from_register_arguments(
                            state.regs[R_A4] as u8,
                            state.regs[R_A0] as usize,
                            state.regs[R_A1] as usize,
                            state.regs[R_A2] as usize,
                            state.regs[R_A3] as usize,
                        );

                        match syscall {
                            Some(s) => ContextSwitchReason::SyscallFired { syscall: s },
                            None => ContextSwitchReason::Fault,
                        }
                    }
                    _ => {
                        // All other exceptions result in faulted state
                        ContextSwitchReason::Fault
                    }
                }
            }
        };
        let new_stack_pointer = state.regs[R_SP];
        (ret, Some(new_stack_pointer as *const u8))
    }

    unsafe fn print_context(
        &self,
        _accessible_memory_start: *const u8,
        _app_brk: *const u8,
        state: &Riscv32iStoredState,
        writer: &mut dyn Write,
    ) {
        let _ = writer.write_fmt(format_args!(
            "\
             \r\n R0 : {:#010X}    R16: {:#010X}\
             \r\n R1 : {:#010X}    R17: {:#010X}\
             \r\n R2 : {:#010X}    R18: {:#010X}\
             \r\n R3 : {:#010X}    R19: {:#010X}\
             \r\n R4 : {:#010X}    R20: {:#010X}\
             \r\n R5 : {:#010X}    R21: {:#010X}\
             \r\n R6 : {:#010X}    R22: {:#010X}\
             \r\n R7 : {:#010X}    R23: {:#010X}\
             \r\n R8 : {:#010X}    R24: {:#010X}\
             \r\n R9 : {:#010X}    R25: {:#010X}\
             \r\n R10: {:#010X}    R26: {:#010X}\
             \r\n R11: {:#010X}    R27: {:#010X}\
             \r\n R12: {:#010X}    R28: {:#010X}\
             \r\n R13: {:#010X}    R29: {:#010X}\
             \r\n R14: {:#010X}    R30: {:#010X}\
             \r\n R15: {:#010X}    R31: {:#010X}\
             \r\n PC : {:#010X}\
             \r\n\
             \r\n mcause: {:#010X} (",
            0,
            state.regs[15],
            state.regs[0],
            state.regs[16],
            state.regs[1],
            state.regs[17],
            state.regs[2],
            state.regs[18],
            state.regs[3],
            state.regs[19],
            state.regs[4],
            state.regs[20],
            state.regs[5],
            state.regs[21],
            state.regs[6],
            state.regs[22],
            state.regs[7],
            state.regs[23],
            state.regs[8],
            state.regs[24],
            state.regs[9],
            state.regs[25],
            state.regs[10],
            state.regs[26],
            state.regs[11],
            state.regs[27],
            state.regs[12],
            state.regs[28],
            state.regs[13],
            state.regs[29],
            state.regs[14],
            state.regs[30],
            state.pc,
            state.mcause,
        ));
        crate::print_mcause(mcause::Trap::from(state.mcause as usize), writer);
        let _ = writer.write_fmt(format_args!(
            ")\
             \r\n mtval:  {:#010X}\
             \r\n\r\n",
            state.mtval,
        ));
    }

    fn store_context(
        &self,
        state: &Riscv32iStoredState,
        out: &mut [u8],
    ) -> Result<usize, ErrorCode> {
        const U32_SZ: usize = size_of::<usize>();
        if out.len() >= size_of::<Riscv32iStoredState>() + METADATA_LEN * U32_SZ {
            write_u32_to_u8_slice(VERSION, out, VERSION_IDX);
            write_u32_to_u8_slice(STORED_STATE_SIZE, out, SIZE_IDX);
            write_u32_to_u8_slice(u32::from_le_bytes(TAG), out, TAG_IDX);
            write_u32_to_u8_slice(state.pc, out, PC_IDX);
            write_u32_to_u8_slice(state.mcause, out, MCAUSE_IDX);
            write_u32_to_u8_slice(state.mtval, out, MTVAL_IDX);
            for (i, v) in state.regs.iter().enumerate() {
                write_u32_to_u8_slice(*v, out, REGS_IDX + i);
            }
            // +3 for pc, mcause, mtval
            Ok((state.regs.len() + 3 + METADATA_LEN) * U32_SZ)
        } else {
            Err(ErrorCode::SIZE)
        }
    }
}
