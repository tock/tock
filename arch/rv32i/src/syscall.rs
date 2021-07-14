//! Kernel-userland system call interface for RISC-V architecture.

use core::fmt::Write;

use crate::csr::mcause;
use kernel;
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
        // registers are stored at monotonically increasing indicies
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
        state.regs[R_RA] = state.pc as u32;

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
          // Before switching to the app we need to save the kernel registers to
          // the kernel stack. We then save the stack pointer in the mscratch
          // CSR (0x340) so we can retrieve it after returning to the kernel
          // from the app.
          //
          // A few values get saved to the kernel stack, including an app
          // register temporarily after entering the trap handler. Here is a
          // memory map to make it easier to keep track:
          //
          // ```
          // 34*4(sp):          <- original stack pointer
          // 33*4(sp):
          // 32*4(sp): x31
          // 31*4(sp): x30
          // 30*4(sp): x29
          // 29*4(sp): x28
          // 28*4(sp): x27
          // 27*4(sp): x26
          // 26*4(sp): x25
          // 25*4(sp): x24
          // 24*4(sp): x23
          // 23*4(sp): x22
          // 22*4(sp): x21
          // 21*4(sp): x20
          // 20*4(sp): x19
          // 19*4(sp): x18
          // 18*4(sp): x17
          // 17*4(sp): x16
          // 16*4(sp): x15
          // 15*4(sp): x14
          // 14*4(sp): x13
          // 13*4(sp): x12
          // 12*4(sp): x11
          // 11*4(sp): x10
          // 10*4(sp): x9
          //  9*4(sp): x8
          //  8*4(sp): x7
          //  7*4(sp): x6
          //  6*4(sp): x5
          //  5*4(sp): x4
          //  4*4(sp): x3
          //  3*4(sp): x1
          //  2*4(sp): _return_to_kernel (address to resume after trap)
          //  1*4(sp): *state   (Per-process StoredState struct)
          //  0*4(sp): app s0   <- new stack pointer
          // ```

          addi sp, sp, -34*4  // Move the stack pointer down to make room.

          sw   x1,  3*4(sp)    // Save all of the registers on the kernel stack.
          sw   x3,  4*4(sp)
          sw   x4,  5*4(sp)
          sw   x5,  6*4(sp)
          sw   x6,  7*4(sp)
          sw   x7,  8*4(sp)
          sw   x8,  9*4(sp)
          sw   x9,  10*4(sp)
          sw   x10, 11*4(sp)
          sw   x11, 12*4(sp)
          sw   x12, 13*4(sp)
          sw   x13, 14*4(sp)
          sw   x14, 15*4(sp)
          sw   x15, 16*4(sp)
          sw   x16, 17*4(sp)
          sw   x17, 18*4(sp)
          sw   x18, 19*4(sp)
          sw   x19, 20*4(sp)
          sw   x20, 21*4(sp)
          sw   x21, 22*4(sp)
          sw   x22, 23*4(sp)
          sw   x23, 24*4(sp)
          sw   x24, 25*4(sp)
          sw   x25, 26*4(sp)
          sw   x26, 27*4(sp)
          sw   x27, 28*4(sp)
          sw   x28, 29*4(sp)
          sw   x29, 30*4(sp)
          sw   x30, 31*4(sp)
          sw   x31, 32*4(sp)

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
          // In asm!() we can't use the shorthand `li` pseudo-instruction,
          // as it complains about _return_to_kernel not being a constant in
          // the required range.
          lui  t0, %hi(_return_to_kernel)
          addi t0, t0, %lo(_return_to_kernel)
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
        _return_to_kernel:

          // We have already stored the app registers in the trap handler. We
          // can restore the kernel registers before resuming kernel code.
          lw   x1,  3*4(sp)
          lw   x3,  4*4(sp)
          lw   x4,  5*4(sp)
          lw   x5,  6*4(sp)
          lw   x6,  7*4(sp)
          lw   x7,  8*4(sp)
          lw   x8,  9*4(sp)
          lw   x9,  10*4(sp)
          lw   x10, 11*4(sp)
          lw   x11, 12*4(sp)
          lw   x12, 13*4(sp)
          lw   x13, 14*4(sp)
          lw   x14, 15*4(sp)
          lw   x15, 16*4(sp)
          lw   x16, 17*4(sp)
          lw   x17, 18*4(sp)
          lw   x18, 19*4(sp)
          lw   x19, 20*4(sp)
          lw   x20, 21*4(sp)
          lw   x21, 22*4(sp)
          lw   x22, 23*4(sp)
          lw   x23, 24*4(sp)
          lw   x24, 25*4(sp)
          lw   x25, 26*4(sp)
          lw   x26, 27*4(sp)
          lw   x27, 28*4(sp)
          lw   x28, 29*4(sp)
          lw   x29, 30*4(sp)
          lw   x30, 31*4(sp)
          lw   x31, 32*4(sp)

          addi sp, sp, 34*4   // Reset kernel stack pointer
          ",

          // The register to put the state struct pointer in is not
          // particularly relevant, however we must avoid using t0
          // as that is overwritten prior to being accessed
          // (although stored and later restored) in the assembly
          in("a0") state as *mut Riscv32iStoredState,
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
}
