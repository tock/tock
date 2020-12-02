//! Implementation of the architecture-specific portions of the kernel-userland
//! system call interface.

use core::fmt::Write;
use core::ptr::{copy_nonoverlapping, read_volatile, write_volatile};

use nix::libc;
use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet, Signal};

use crate::{FLASH_POSITION, ORIGINAL_FLASH_POSITION};

/// This is used in the syscall handler. When set to 1 this means the
/// svc_handler was called.
#[no_mangle]
#[used]
static mut SYSCALL_FIRED: usize = 0;

/// This is called in the hard fault handler. When set to 1 this means the hard
/// fault handler was called.
///
/// n.b. If the kernel hard faults, it immediately panic's. This flag is only
/// for handling application hard faults.
#[no_mangle]
#[used]
static mut APP_HARD_FAULT: usize = 0;

/// This is used to verify if the kernel or a process is running
pub static mut RUNNING_KERNEL: bool = true;

/// Posix reserved space
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
const POSIX_RESERVED_LEN: usize = 5;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
const POSIX_REGISTERS_LEN: usize = 23;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
const POSIX_NEXT_USER_PC: usize = 8;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
const POSIX_NEXT_KERNEL_PC: usize = 8;

/// The AMD64 registers
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub type RegsData = [usize; 23];

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub struct PosixContext {
    _reserved: [usize; POSIX_RESERVED_LEN],
    pub(crate) regs: RegsData,
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[repr(usize)]
#[allow(non_snake_case)]
pub enum Regs {
    R8 = 0,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    RDI,
    RSI,
    RBP,
    RBX,
    RDX,
    RAX,
    RCX,
    RSP,
    RIP,
    EFL,
    CSGSFS,
    ERR,
    TRAPNO,
    OLDMASK,
    CR2,
}

impl Regs {
    #[inline]
    pub fn pc() -> Regs {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        Regs::RIP
    }

    #[inline]
    pub fn sp() -> Regs {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        Regs::RSP
    }

    #[inline]
    pub fn r0() -> Regs {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        Regs::RAX
    }

    #[inline]
    pub fn r1() -> Regs {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        Regs::RBX
    }

    #[inline]
    pub fn r2() -> Regs {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        Regs::RCX
    }

    #[inline]
    pub fn r3() -> Regs {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        Regs::RDX
    }

    #[inline]
    pub fn r4() -> Regs {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        Regs::R10
    }
}

impl PosixContext {
    pub fn new(ip: usize, sp: usize) -> PosixContext {
        let mut context = PosixContext {
            _reserved: [0; POSIX_RESERVED_LEN],
            regs: RegsData::default(),
        };
        context.write_register(Regs::pc(), ip);
        context.write_register(Regs::sp(), sp);
        context
    }

    pub const fn for_kernel() -> PosixContext {
        PosixContext {
            _reserved: [0; POSIX_RESERVED_LEN],
            regs: [0; POSIX_REGISTERS_LEN],
        }
    }

    #[inline]
    pub fn read_register(&self, reg: Regs) -> usize {
        self.regs[reg as usize]
    }

    #[inline]
    pub fn write_register(&mut self, reg: Regs, val: usize) {
        self.regs[reg as usize] = val
    }

    pub fn first_run(&mut self, kernel_context: &Self) {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            self.write_register(Regs::CSGSFS, kernel_context.read_register(Regs::CSGSFS));
            self.write_register(Regs::ERR, kernel_context.read_register(Regs::ERR));
            self.write_register(Regs::TRAPNO, kernel_context.read_register(Regs::TRAPNO));
            self.write_register(Regs::OLDMASK, kernel_context.read_register(Regs::OLDMASK));
            self.write_register(Regs::CR2, kernel_context.read_register(Regs::CR2));
        }
    }
}

static mut KERNEL_CONTEXT: PosixContext = PosixContext::for_kernel();
pub static mut CURRENT_PROCESS: PosixStoredState = PosixStoredState::empty_user();

#[no_mangle]
#[inline(never)]
pub unsafe fn switch_context_kernel(user_state: &mut PosixStoredState, context: &mut PosixContext) {
    // save application context
    std::ptr::copy_nonoverlapping(
        context as *mut PosixContext as *mut u8,
        &mut user_state.context as *mut PosixContext as *mut u8,
        std::mem::size_of::<PosixContext>(),
    );
    // restore kernel context
    // offset the return address
    std::ptr::copy_nonoverlapping(
        &mut KERNEL_CONTEXT as *mut PosixContext as *mut u8,
        context as *mut PosixContext as *mut u8,
        std::mem::size_of::<PosixContext>(),
    );
}
#[no_mangle]
// uncomment for denug
// #[inline(never)]
unsafe fn switched_context_kernel() {
    write_volatile(&mut RUNNING_KERNEL, true);
}

// used for debugging
#[no_mangle]
// uncomment for denug
// #[inline(never)]
unsafe fn switched_context_user() {
    write_volatile(&mut RUNNING_KERNEL, false);
}

#[no_mangle]
#[inline(never)]
pub unsafe fn switch_context_user(user_state: &mut PosixStoredState, context: &mut PosixContext) {
    // save kernel context
    context.write_register(
        Regs::pc(),
        context.read_register(Regs::pc()) + POSIX_NEXT_KERNEL_PC,
    );
    std::ptr::copy_nonoverlapping(
        context as *mut PosixContext as *mut u8,
        &mut KERNEL_CONTEXT as *mut PosixContext as *mut u8,
        std::mem::size_of::<PosixContext>(),
    );
    // restore application context
    // offset the return address

    std::ptr::copy_nonoverlapping(
        &mut user_state.context as *mut PosixContext as *mut u8,
        context as *mut PosixContext as *mut u8,
        std::mem::size_of::<PosixContext>(),
    );

    if user_state.first_run {
        std::ptr::copy_nonoverlapping(
            &KERNEL_CONTEXT._reserved.as_ptr(),
            &mut context._reserved.as_ptr(),
            KERNEL_CONTEXT._reserved.len(),
        );

        context.first_run(&KERNEL_CONTEXT);
    }

    context.write_register(Regs::pc(), user_state.sys_pc);
    context.write_register(Regs::sp(), user_state.stack_pointer);
}

#[inline(never)]
#[no_mangle]
fn switch_to_user() {
    unsafe {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        llvm_asm!(
            "
            mov rax, 0x515CA11F
            mov [0], rax
            "
            : : : "rax" : "volatile", "intel"
        )
    }
}

extern "C" fn handle_sigsegv(
    _signal: libc::c_int,
    _siginfo: *mut libc::siginfo_t,
    signal_context: *mut libc::c_void,
) {
    let mut context = unsafe { &mut *(signal_context as *mut usize as *mut PosixContext) };
    let key: usize = context.read_register(Regs::r0());
    context.write_register(Regs::r0(), key & 0x7);
    if (key & 0x515CA110) == 0x515CA110 {
        let action = key & 0x7;
        let direction = (key & 0xF) >> 3;
        if direction == 1 {
            // to user space
            unsafe {
                switch_context_user(&mut CURRENT_PROCESS, &mut context);
                switched_context_user();
            }
        } else {
            // to kernel space
            // println!("action {}", action);
            match action {
                0..=4 => unsafe {
                    SYSCALL_FIRED = 1;
                    context.write_register(
                        Regs::pc(),
                        context.read_register(Regs::pc()) + POSIX_NEXT_USER_PC,
                    );
                    switch_context_kernel(&mut CURRENT_PROCESS, &mut context);
                    switched_context_kernel();
                },
                _ => {
                    // fault syscall, consider it a fault
                    unsafe {
                        APP_HARD_FAULT = 1;
                    }
                }
            }
        }
    } else {
        // a fault
        unsafe {
            APP_HARD_FAULT = 1;
        }
    }
}

pub unsafe fn init() {
    let handler = SigHandler::SigAction(handle_sigsegv);
    let sigaction = SigAction::new(handler, SaFlags::empty(), SigSet::all());
    signal::sigaction(Signal::SIGSEGV, &sigaction).unwrap();
}

/// This holds all of the state that the kernel must keep for the process when
/// the process is not executing.
#[derive(Default, Copy, Clone)]
pub struct PosixStoredState {
    context: PosixContext,
    first_run: bool,
    stack_pointer: usize,
    sys_pc: usize,
    yield_pc: usize,
}

impl PosixStoredState {
    pub const fn empty_user() -> PosixStoredState {
        PosixStoredState {
            context: PosixContext::for_kernel(),
            first_run: false,
            stack_pointer: 0,
            sys_pc: 0,
            yield_pc: 0,
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
    type StoredState = PosixStoredState;

    unsafe fn initialize_process(
        &self,
        stack_pointer: *const usize,
        _stack_size: usize,
        state: &mut Self::StoredState,
    ) -> Result<*const usize, ()> {
        // We need to initialize the stored state for the process here. This
        // initialization can be called multiple times for a process, for
        // example if the process is restarted.
        state.first_run = true;
        state.sys_pc = 0;
        state.yield_pc = 0;

        // Allocate the kernel frame
        Ok((stack_pointer as *mut usize).offset(0))
    }

    unsafe fn set_syscall_return_value(
        &self,
        _stack_pointer: *const usize,
        state: &mut Self::StoredState,
        return_value: isize,
    ) {
        // set the R0 (RAX, EAX, ...) register
        state
            .context
            .write_register(Regs::r0(), return_value as usize);
    }

    unsafe fn set_process_function(
        &self,
        stack_pointer: *const usize,
        _remaining_stack_memory: usize,
        state: &mut PosixStoredState,
        callback: kernel::procs::FunctionCall,
    ) -> Result<*mut usize, *mut usize> {
        let mut stack_bottom = stack_pointer as *mut usize;
        state.sys_pc = callback.pc;

        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            // add the yield RIP value to the stack
            std::ptr::write_volatile(stack_bottom.offset(-1), state.yield_pc);
            stack_bottom = stack_bottom.offset(-1);
            // AMD 64 sets the parameters in RDI, RSI, RDX and RCX
            state.context.write_register(Regs::RDI, callback.argument0);
            state.context.write_register(Regs::RSI, callback.argument1);
            state.context.write_register(Regs::RDX, callback.argument2);
            state.context.write_register(Regs::RCX, callback.argument3);
        }

        // Copy .text, .rodata and .data from originial flash
        // as this might be a process restart and data in flash
        // is already relocated
        if state.first_run {
            let offset = callback.argument0 - FLASH_POSITION;
            let data_offset = read_volatile((callback.argument0 + 12) as *const u32);
            let data_len = read_volatile((callback.argument0 + 20) as *const u32);
            copy_nonoverlapping(
                (ORIGINAL_FLASH_POSITION + offset) as *const u8,
                (FLASH_POSITION + offset) as *mut u8,
                (data_offset + data_len) as usize,
            );
        }

        Ok(stack_bottom)
    }

    unsafe fn switch_to_process(
        &self,
        stack_pointer: *const usize,
        state: &mut PosixStoredState,
    ) -> (*mut usize, kernel::syscall::ContextSwitchReason) {
        state.stack_pointer = stack_pointer as usize;
        CURRENT_PROCESS = *state;
        switch_to_user();
        *state = CURRENT_PROCESS;
        state.first_run = false;

        state.sys_pc = state.context.read_register(Regs::pc());
        let new_stack_pointer = state.context.read_register(Regs::sp()) as *const usize;

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
            state.yield_pc = state.context.read_register(Regs::pc());

            // Get the syscall arguments and return them along with the syscall.
            // It's possible the app did something invalid, in which case we put
            // the app in the fault state.
            let syscall_number = (state.context.read_register(Regs::r0()) & 0x7) as u8;
            let r0 = state.context.read_register(Regs::r1());
            let r1 = state.context.read_register(Regs::r2());
            let r2 = state.context.read_register(Regs::r3());
            let r3 = state.context.read_register(Regs::r4());

            // Use the helper function to convert these raw values into a Tock
            // `Syscall` type.
            let syscall = kernel::syscall::arguments_to_syscall(syscall_number, r0, r1, r2, r3);

            match syscall {
                Some(s) => kernel::syscall::ContextSwitchReason::SyscallFired { syscall: s },
                None => kernel::syscall::ContextSwitchReason::Fault,
            }
        } else {
            // If none of the above cases are true its because the process was interrupted by an
            // ISR for a hardware event
            kernel::syscall::ContextSwitchReason::Interrupted
        };
        // println!("switch {:?}", switch_reason);
        (new_stack_pointer as *mut usize, switch_reason)
    }

    unsafe fn print_context(
        &self,
        _stack_pointer: *const usize,
        state: &PosixStoredState,
        writer: &mut dyn Write,
    ) {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            let flags = state.context.read_register(Regs::EFL);

            let _ = writer.write_fmt(format_args!(
                "\
                \r\n  RAX : {:#018X}    R9  : {:#018X}\
                \r\n  RBX : {:#018X}    R10 : {:#018X}\
                \r\n  RCX : {:#018X}    R11 : {:#018X}\
                \r\n  RDX : {:#018X}    R12 : {:#018X}\
                \r\n  RSI : {:#018X}    R13 : {:#018X}\
                \r\n  RDI : {:#018X}    R14 : {:#018X}\
                \r\n  R8  : {:#018X}    R15 : {:#018X}\
                \r\n  RBP : {:#018X} (Static Base Register)\
                \r\n  RSP : {:#018X} (Process Stack Pointer)\
                \r\n  RIP : {:#018X}\
                \r\n YRIP : {:#018X}\
                \r\n",
                state.context.read_register(Regs::RAX),
                state.context.read_register(Regs::R9),
                state.context.read_register(Regs::RBX),
                state.context.read_register(Regs::R10),
                state.context.read_register(Regs::RCX),
                state.context.read_register(Regs::R11),
                state.context.read_register(Regs::RDX),
                state.context.read_register(Regs::R12),
                state.context.read_register(Regs::RSI),
                state.context.read_register(Regs::R13),
                state.context.read_register(Regs::RDI),
                state.context.read_register(Regs::R14),
                state.context.read_register(Regs::R8),
                state.context.read_register(Regs::R15),
                state.context.read_register(Regs::RBP),
                state.context.read_register(Regs::RSP),
                state.context.read_register(Regs::RIP),
                state.yield_pc,
            ));
            let _ = writer.write_fmt(format_args!(
                "\
                \r\n FLAGS: C: {} P: {} A: {} Z: {} S: {}\
                \r\n        T: {} I: {} D: {} O: {}",
                (flags & 0x1),
                (flags >> 2) & 0x1,
                (flags >> 4) & 0x1,
                (flags >> 6) & 0x1,
                (flags >> 7) & 0x1,
                (flags >> 8) & 0x1,
                (flags >> 9) & 0x1,
                (flags >> 10) & 0x1,
                (flags >> 11) & 0x1,
            ));
        }
    }
}
