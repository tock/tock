//! Implementation of the architecture-specific portions of the kernel-userland
//! system call interface.

use core::ptr::{read_volatile, write_volatile};

use kernel;

/// This is used in the syscall handler. When set to 1 this means the
/// svc_handler was called. Marked `pub` because it is used in the cortex-m*
/// specific handler.
#[no_mangle]
#[used]
pub static mut SYSCALL_FIRED: usize = 0;

/// This is called in the hard fault handler. When set to 1 this means the hard
/// fault handler was called. Marked `pub` because it is used in the cortex-m*
/// specific handler.
#[no_mangle]
#[used]
pub static mut APP_FAULT: usize = 0;

#[allow(improper_ctypes)]
extern "C" {
    pub fn switch_to_user(user_stack: *const u8, process_regs: &mut [usize; 8]) -> *mut u8;
}

/// This holds all of the state that the kernel must keep for the process when
/// the process is not executing.
pub struct CortexMStoredState {
    pub r4: usize,
    pub r5: usize,
    pub r6: usize,
    pub r7: usize,
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
    yield_pc: usize,
    psr: usize,
}

// Need a custom define for `default()` so we can set the initial PSR value.
impl Default for CortexMStoredState {
    fn default() -> CortexMStoredState {
        CortexMStoredState {
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            yield_pc: 0,
            // Set the Thumb bit and clear everything else
            psr: 0x01000000,
        }
    }
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

    unsafe fn get_and_reset_context_switch_reason(&self) -> kernel::syscall::ContextSwitchReason {
        let app_fault = read_volatile(&APP_FAULT);
        // We are free to reset this immediately as this function will only get
        // called once.
        write_volatile(&mut APP_FAULT, 0);

        // Check to see if the svc_handler was called and the process called a
        // syscall.
        let syscall_fired = read_volatile(&SYSCALL_FIRED);
        write_volatile(&mut SYSCALL_FIRED, 0);

        if app_fault == 1 {
            // APP_FAULT takes priority. This means we hit the hardfault handler
            // and this process faulted.
            kernel::syscall::ContextSwitchReason::Fault
        } else if syscall_fired == 1 {
            kernel::syscall::ContextSwitchReason::SyscallFired
        } else {
            kernel::syscall::ContextSwitchReason::TimesliceExpired
        }
    }

    /// Get the syscall that the process called.
    unsafe fn get_syscall(&self, stack_pointer: *const usize) -> Option<kernel::syscall::Syscall> {
        // Get the four values that are passed with the syscall.
        let r0 = read_volatile(stack_pointer.offset(0));
        let r1 = read_volatile(stack_pointer.offset(1));
        let r2 = read_volatile(stack_pointer.offset(2));
        let r3 = read_volatile(stack_pointer.offset(3));

        // Get the actual SVC number.
        let pcptr = read_volatile((stack_pointer as *const *const u16).offset(6));
        let svc_instr = read_volatile(pcptr.offset(-1));
        let svc_num = (svc_instr & 0xff) as u8;
        match svc_num {
            0 => Some(kernel::syscall::Syscall::YIELD),
            1 => Some(kernel::syscall::Syscall::SUBSCRIBE {
                driver_number: r0,
                subdriver_number: r1,
                callback_ptr: r2 as *mut (),
                appdata: r3,
            }),
            2 => Some(kernel::syscall::Syscall::COMMAND {
                driver_number: r0,
                subdriver_number: r1,
                arg0: r2,
                arg1: r3,
            }),
            3 => Some(kernel::syscall::Syscall::ALLOW {
                driver_number: r0,
                subdriver_number: r1,
                allow_address: r2 as *mut u8,
                allow_size: r3,
            }),
            4 => Some(kernel::syscall::Syscall::MEMOP {
                operand: r0,
                arg0: r1,
            }),
            _ => None,
        }
    }

    unsafe fn set_syscall_return_value(&self, stack_pointer: *const usize, return_value: isize) {
        // For the Cortex-M arch we set this in the same place that r0 was
        // passed.
        let sp = stack_pointer as *mut isize;
        write_volatile(sp, return_value);
    }

    unsafe fn pop_syscall_stack_frame(
        &self,
        stack_pointer: *const usize,
        state: &mut CortexMStoredState,
    ) -> *mut usize {
        state.yield_pc = read_volatile(stack_pointer.offset(6));
        state.psr = read_volatile(stack_pointer.offset(7));
        (stack_pointer as *mut usize).offset(8)
    }

    unsafe fn push_function_call(
        &self,
        stack_pointer: *const usize,
        remaining_stack_memory: usize,
        callback: kernel::procs::FunctionCall,
        state: &CortexMStoredState,
    ) -> Result<*mut usize, *mut usize> {
        // We need 32 bytes to add this frame. Ensure that there are 32 bytes
        // available on the stack.
        if remaining_stack_memory < 32 {
            // Not enough room on the stack to add a frame. Return an error
            // and where the stack would be to help with debugging.
            Err((stack_pointer as *mut usize).offset(-8))
        } else {
            // Fill in initial stack expected by SVC handler
            // Top minus 8 u32s for r0-r3, r12, lr, pc and xPSR
            let stack_bottom = (stack_pointer as *mut usize).offset(-8);
            write_volatile(stack_bottom.offset(7), state.psr);
            write_volatile(stack_bottom.offset(6), callback.pc | 1);

            // Set the LR register to the saved PC so the callback returns to
            // wherever wait was called. Set lowest bit to one because of THUMB
            // instruction requirements.
            write_volatile(stack_bottom.offset(5), state.yield_pc | 0x1);
            write_volatile(stack_bottom, callback.argument0);
            write_volatile(stack_bottom.offset(1), callback.argument1);
            write_volatile(stack_bottom.offset(2), callback.argument2);
            write_volatile(stack_bottom.offset(3), callback.argument3);

            Ok(stack_bottom)
        }
    }

    unsafe fn switch_to_process(
        &self,
        stack_pointer: *const usize,
        state: &mut CortexMStoredState,
    ) -> *mut usize {
        switch_to_user(
            stack_pointer as *const u8,
            &mut *(state as *mut CortexMStoredState as *mut [usize; 8]),
        ) as *mut usize
    }
}
