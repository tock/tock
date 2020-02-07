//! Implementation of the architecture-specific portions of the kernel-userland
//! system call interface.

use core::fmt::Write;
use core::ptr::{read_volatile, write_volatile};

/// This is used in the syscall handler. When set to 1 this means the
/// svc_handler was called. Marked `pub` because it is used in the cortex-m*
/// specific handler.
#[no_mangle]
#[used]
pub static mut SYSCALL_FIRED: usize = 0;

/// This is called in the hard fault handler. When set to 1 this means the hard
/// fault handler was called. Marked `pub` because it is used in the cortex-m*
/// specific handler.
///
/// n.b. If the kernel hard faults, it immediately panic's. This flag is only
/// for handling application hard faults.
#[no_mangle]
#[used]
pub static mut APP_HARD_FAULT: usize = 0;

/// This is called in the systick handler. When set to 1 this means the process
/// exceeded its timeslice. Marked `pub` because it is used in the cortex-m*
/// specific handler.
#[no_mangle]
#[used]
pub static mut SYSTICK_EXPIRED: usize = 0;

/// This is used in the hardfault handler. When an app faults, the hardfault
/// handler stores the value of the SCB registers in this static array. This
/// makes them available to be displayed in a diagnostic fault message.
#[no_mangle]
#[used]
pub static mut SCB_REGISTERS: [u32; 5] = [0; 5];

#[allow(improper_ctypes)]
extern "C" {
    pub fn switch_to_user(user_stack: *const usize, process_regs: &mut [usize; 8]) -> *const usize;
}

/// This holds all of the state that the kernel must keep for the process when
/// the process is not executing.
#[derive(Copy, Clone, Default)]
pub struct CortexMStoredState {
    regs: [usize; 8],
    yield_pc: usize,
    psr: usize,
}

/// Implementation of the `UserspaceKernelBoundary` for the Cortex-M non-floating point
/// architecture.
pub struct SysCall();

impl SysCall {
    pub const unsafe fn new() -> SysCall {
        SysCall()
    }
}

impl kernel::syscall::UserspaceKernelBoundary for SysCall {
    type StoredState = CortexMStoredState;

    unsafe fn initialize_process(
        &self,
        stack_pointer: *const usize,
        stack_size: usize,
        state: &mut Self::StoredState,
    ) -> Result<*const usize, ()> {
        // We need to initialize the stored state for the process here. This
        // initialization can be called multiple times for a process, for
        // example if the process is restarted.
        state.regs.iter_mut().for_each(|x| *x = 0);
        state.yield_pc = 0;
        state.psr = 0x01000000; // Set the Thumb bit and clear everything else.

        // The first time a process runs it has no stack and we have to create
        // a new stack frame for the svc handler to have "returned from".

        // Space for 8 u32s: r0-r3, r12, lr, pc, and xPSR
        let svc_frame_size = 32;

        // Make sure there's enough room on the stack for the initial kernel frame
        if stack_size < svc_frame_size {
            // Not enough room on the stack to add a frame.
            return Err(());
        }

        // Allocate the kernel frame
        Ok((stack_pointer as *mut usize).offset(-8))
    }

    unsafe fn set_syscall_return_value(
        &self,
        stack_pointer: *const usize,
        _state: &mut Self::StoredState,
        return_value: isize,
    ) {
        // For the Cortex-M arch we set this in the same place that r0 was
        // passed.
        let sp = stack_pointer as *mut isize;
        write_volatile(sp, return_value);
    }

    /// When the process calls `svc` to enter the kernel, the hardware
    /// automatically pushes a stack frame that will be unstacked when the
    /// kernel returns to the process. In the special case of process startup,
    /// `initialize_new_process` sets up an empty stack frame and stored state
    /// as if an `svc` had been called.
    ///
    /// Here, we modify this stack frame such that the process resumes at the
    /// beginning of the callback function that we want the process to run. We
    /// place the originally intended return addess in the link register so
    /// that when the function completes execution continues.
    ///
    /// In effect, this converts `svc` into `bl callback`.
    unsafe fn set_process_function(
        &self,
        stack_pointer: *const usize,
        _remaining_stack_memory: usize,
        state: &mut CortexMStoredState,
        callback: kernel::procs::FunctionCall,
    ) -> Result<*mut usize, *mut usize> {
        // Notes:
        //  - Instruction addresses require `|1` to indicate thumb code
        //  - Stack offset 4 is R12, which the syscall interface ignores
        let stack_bottom = stack_pointer as *mut usize;
        write_volatile(stack_bottom.offset(7), state.psr); //......... -> APSR
        write_volatile(stack_bottom.offset(6), callback.pc | 1); //... -> PC
        write_volatile(stack_bottom.offset(5), state.yield_pc | 1); // -> LR
        write_volatile(stack_bottom.offset(3), callback.argument3); // -> R3
        write_volatile(stack_bottom.offset(2), callback.argument2); // -> R2
        write_volatile(stack_bottom.offset(1), callback.argument1); // -> R1
        write_volatile(stack_bottom.offset(0), callback.argument0); // -> R0

        Ok(stack_bottom)
    }

    unsafe fn switch_to_process(
        &self,
        stack_pointer: *const usize,
        state: &mut CortexMStoredState,
    ) -> (*mut usize, kernel::syscall::ContextSwitchReason) {
        let new_stack_pointer = switch_to_user(stack_pointer, &mut state.regs);

        // Determine why this returned and the process switched back to the
        // kernel.

        // Check to see if the fault handler was called while the process was
        // running.
        let app_fault = read_volatile(&APP_HARD_FAULT);
        write_volatile(&mut APP_HARD_FAULT, 0);

        // Check to see if the svc_handler was called and the process called a
        // syscall.
        let syscall_fired = read_volatile(&SYSCALL_FIRED);
        write_volatile(&mut SYSCALL_FIRED, 0);

        // Check to see if the systick timer for the process expired.
        let systick_expired = read_volatile(&SYSTICK_EXPIRED);
        write_volatile(&mut SYSTICK_EXPIRED, 0);

        // Now decide the reason based on which flags were set.
        let switch_reason = if app_fault == 1 {
            // APP_HARD_FAULT takes priority. This means we hit the hardfault
            // handler and this process faulted.
            kernel::syscall::ContextSwitchReason::Fault
        } else if syscall_fired == 1 {
            // Save these fields after a syscall. If this is a synchronous
            // syscall (i.e. we return a value to the app immediately) then this
            // will have no effect. If we are doing something like `yield()`,
            // however, then we need to have this state.
            state.yield_pc = read_volatile(new_stack_pointer.offset(6));
            state.psr = read_volatile(new_stack_pointer.offset(7));

            // Get the syscall arguments and return them along with the syscall.
            // It's possible the app did something invalid, in which case we put
            // the app in the fault state.
            let r0 = read_volatile(new_stack_pointer.offset(0));
            let r1 = read_volatile(new_stack_pointer.offset(1));
            let r2 = read_volatile(new_stack_pointer.offset(2));
            let r3 = read_volatile(new_stack_pointer.offset(3));

            // Get the actual SVC number.
            let pcptr = read_volatile((new_stack_pointer as *const *const u16).offset(6));
            let svc_instr = read_volatile(pcptr.offset(-1));
            let svc_num = (svc_instr & 0xff) as u8;

            // Use the helper function to convert these raw values into a Tock
            // `Syscall` type.
            let syscall = kernel::syscall::arguments_to_syscall(svc_num, r0, r1, r2, r3);

            match syscall {
                Some(s) => kernel::syscall::ContextSwitchReason::SyscallFired { syscall: s },
                None => kernel::syscall::ContextSwitchReason::Fault,
            }
        } else if systick_expired == 1 {
            kernel::syscall::ContextSwitchReason::TimesliceExpired
        } else {
            // If none of the above cases are true its because the process was interrupted by an
            // ISR for a hardware event
            kernel::syscall::ContextSwitchReason::Interrupted
        };

        (new_stack_pointer as *mut usize, switch_reason)
    }

    unsafe fn print_context(
        &self,
        stack_pointer: *const usize,
        state: &CortexMStoredState,
        writer: &mut dyn Write,
    ) {
        let r0 = read_volatile(stack_pointer.offset(0));
        let r1 = read_volatile(stack_pointer.offset(1));
        let r2 = read_volatile(stack_pointer.offset(2));
        let r3 = read_volatile(stack_pointer.offset(3));
        let r12 = read_volatile(stack_pointer.offset(4));
        let lr = read_volatile(stack_pointer.offset(5));
        let pc = read_volatile(stack_pointer.offset(6));
        let xpsr = read_volatile(stack_pointer.offset(7));

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
                "!!ERROR - Cortex M Thumb only!"
            },
        ));
    }
}
