//! Implementation of the architecture-specific portions of the kernel-userland
//! system call interface.

use core::fmt::Write;
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
#[derive(Copy, Clone)]
pub struct CortexMStoredState {
    regs: [usize; 8],
    yield_pc: usize,
    psr: usize,
}

// Need a custom define for `default()` so we can set the initial PSR value.
impl Default for CortexMStoredState {
    fn default() -> CortexMStoredState {
        CortexMStoredState {
            regs: [0; 8],
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
            kernel::syscall::ContextSwitchReason::SyscallFired
        } else if systick_expired == 1 {
            kernel::syscall::ContextSwitchReason::TimesliceExpired
        } else {
            // Desired: If something else happened, which shouldn't, we fallback
            // to this process having faulted.
            //
            // Currently: Defaulting to `Fault` is causing the first app to
            // crash immediately. While that can be sorted, default to
            // essentially a no-op so that Tock works again. Also this is (I
            // think) the old behavior (before #1113).
            kernel::syscall::ContextSwitchReason::Interrupted
        };

        (new_stack_pointer as *mut usize, switch_reason)
    }

    unsafe fn fault_fmt(&self, writer: &mut Write) {
        let _ccr = SCB_REGISTERS[0];
        let cfsr = SCB_REGISTERS[1];
        let hfsr = SCB_REGISTERS[2];
        let mmfar = SCB_REGISTERS[3];
        let bfar = SCB_REGISTERS[4];

        let iaccviol = (cfsr & 0x01) == 0x01;
        let daccviol = (cfsr & 0x02) == 0x02;
        let munstkerr = (cfsr & 0x08) == 0x08;
        let mstkerr = (cfsr & 0x10) == 0x10;
        let mlsperr = (cfsr & 0x20) == 0x20;
        let mmfarvalid = (cfsr & 0x80) == 0x80;

        let ibuserr = ((cfsr >> 8) & 0x01) == 0x01;
        let preciserr = ((cfsr >> 8) & 0x02) == 0x02;
        let impreciserr = ((cfsr >> 8) & 0x04) == 0x04;
        let unstkerr = ((cfsr >> 8) & 0x08) == 0x08;
        let stkerr = ((cfsr >> 8) & 0x10) == 0x10;
        let lsperr = ((cfsr >> 8) & 0x20) == 0x20;
        let bfarvalid = ((cfsr >> 8) & 0x80) == 0x80;

        let undefinstr = ((cfsr >> 16) & 0x01) == 0x01;
        let invstate = ((cfsr >> 16) & 0x02) == 0x02;
        let invpc = ((cfsr >> 16) & 0x04) == 0x04;
        let nocp = ((cfsr >> 16) & 0x08) == 0x08;
        let unaligned = ((cfsr >> 16) & 0x100) == 0x100;
        let divbysero = ((cfsr >> 16) & 0x200) == 0x200;

        let vecttbl = (hfsr & 0x02) == 0x02;
        let forced = (hfsr & 0x40000000) == 0x40000000;

        let _ = writer.write_fmt(format_args!("\r\n---| Fault Status |---\r\n"));

        if iaccviol {
            let _ = writer.write_fmt(format_args!(
                "Instruction Access Violation:       {}\r\n",
                iaccviol
            ));
        }
        if daccviol {
            let _ = writer.write_fmt(format_args!(
                "Data Access Violation:              {}\r\n",
                daccviol
            ));
        }
        if munstkerr {
            let _ = writer.write_fmt(format_args!(
                "Memory Management Unstacking Fault: {}\r\n",
                munstkerr
            ));
        }
        if mstkerr {
            let _ = writer.write_fmt(format_args!(
                "Memory Management Stacking Fault:   {}\r\n",
                mstkerr
            ));
        }
        if mlsperr {
            let _ = writer.write_fmt(format_args!(
                "Memory Management Lazy FP Fault:    {}\r\n",
                mlsperr
            ));
        }

        if ibuserr {
            let _ = writer.write_fmt(format_args!(
                "Instruction Bus Error:              {}\r\n",
                ibuserr
            ));
        }
        if preciserr {
            let _ = writer.write_fmt(format_args!(
                "Precise Data Bus Error:             {}\r\n",
                preciserr
            ));
        }
        if impreciserr {
            let _ = writer.write_fmt(format_args!(
                "Imprecise Data Bus Error:           {}\r\n",
                impreciserr
            ));
        }
        if unstkerr {
            let _ = writer.write_fmt(format_args!(
                "Bus Unstacking Fault:               {}\r\n",
                unstkerr
            ));
        }
        if stkerr {
            let _ = writer.write_fmt(format_args!(
                "Bus Stacking Fault:                 {}\r\n",
                stkerr
            ));
        }
        if lsperr {
            let _ = writer.write_fmt(format_args!(
                "Bus Lazy FP Fault:                  {}\r\n",
                lsperr
            ));
        }
        if undefinstr {
            let _ = writer.write_fmt(format_args!(
                "Undefined Instruction Usage Fault:  {}\r\n",
                undefinstr
            ));
        }
        if invstate {
            let _ = writer.write_fmt(format_args!(
                "Invalid State Usage Fault:          {}\r\n",
                invstate
            ));
        }
        if invpc {
            let _ = writer.write_fmt(format_args!(
                "Invalid PC Load Usage Fault:        {}\r\n",
                invpc
            ));
        }
        if nocp {
            let _ = writer.write_fmt(format_args!(
                "No Coprocessor Usage Fault:         {}\r\n",
                nocp
            ));
        }
        if unaligned {
            let _ = writer.write_fmt(format_args!(
                "Unaligned Access Usage Fault:       {}\r\n",
                unaligned
            ));
        }
        if divbysero {
            let _ = writer.write_fmt(format_args!(
                "Divide By Zero:                     {}\r\n",
                divbysero
            ));
        }

        if vecttbl {
            let _ = writer.write_fmt(format_args!(
                "Bus Fault on Vector Table Read:     {}\r\n",
                vecttbl
            ));
        }
        if forced {
            let _ = writer.write_fmt(format_args!(
                "Forced Hard Fault:                  {}\r\n",
                forced
            ));
        }

        if mmfarvalid {
            let _ = writer.write_fmt(format_args!(
                "Faulting Memory Address:            {:#010X}\r\n",
                mmfar
            ));
        }
        if bfarvalid {
            let _ = writer.write_fmt(format_args!(
                "Bus Fault Address:                  {:#010X}\r\n",
                bfar
            ));
        }

        if cfsr == 0 && hfsr == 0 {
            let _ = writer.write_fmt(format_args!("No faults detected.\r\n"));
        } else {
            let _ = writer.write_fmt(format_args!(
                "Fault Status Register (CFSR):       {:#010X}\r\n",
                cfsr
            ));
            let _ = writer.write_fmt(format_args!(
                "Hard Fault Status Register (HFSR):  {:#010X}\r\n",
                hfsr
            ));
        }
    }

    unsafe fn process_detail_fmt(
        &self,
        stack_pointer: *const usize,
        state: &CortexMStoredState,
        writer: &mut Write,
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
             \r\n       ThumbBit {} {}",
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
