//! Shared implementations for ARM Cortex-M3 MCUs.

#![crate_name = "cortexm3"]
#![crate_type = "rlib"]
#![no_std]

pub mod mpu {
    pub type MPU = cortexm::mpu::MPU<8>;
}

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m3.
pub use cortexm::support;

pub use cortexm::generic_isr_arm_v7m as generic_isr;
pub use cortexm::hard_fault_handler_arm_v7m as hard_fault_handler;
pub use cortexm::nvic;
pub use cortexm::print_cortexm_state as print_cortexm3_state;
pub use cortexm::scb;
pub use cortexm::svc_handler_arm_v7m as svc_handler;
pub use cortexm::syscall;
pub use cortexm::systick;
pub use cortexm::systick_handler_arm_v7m as systick_handler;

/// Assembly function called from `UserspaceKernelBoundary` to switch to an
/// an application. This handles storing and restoring application state before
/// and after the switch.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn switch_to_user_arm_v7m(
    mut user_stack: *const usize,
    process_regs: &mut [usize; 8],
) -> *const usize {
    llvm_asm!(
        "
    // The arguments passed in are:
    // - `r0` is the top of the user stack
    // - `r1` is a reference to `CortexMStoredState.regs`

    // Load bottom of stack into Process Stack Pointer.
    msr psp, $0

    // Load non-hardware-stacked registers from the process stored state. Ensure
    // that $2 is stored in a callee saved register.
    ldmia $2, {r4-r11}

    // SWITCH
    svc 0xff   // It doesn't matter which SVC number we use here as it has no
               // defined meaning for the Cortex-M syscall interface. Data being
               // returned from a syscall is transfered on the app's stack.

    // When execution returns here we have switched back to the kernel from the
    // application.

    // Push non-hardware-stacked registers into the saved state for the
    // application.
    stmia $2, {r4-r11}

    // Update the user stack pointer with the current value after the
    // application has executed.
    mrs $0, PSP   // r0 = PSP"
    : "={r0}"(user_stack)
    : "{r0}"(user_stack), "{r1}"(process_regs)
    : "r4","r5","r6","r8","r9","r10","r11" : "volatile" );
    user_stack
}

/// Provide a `switch_to_user` function with exactly that name for syscall.rs.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn switch_to_user(
    mut user_stack: *const usize,
    process_regs: &mut [usize; 8],
) -> *const usize {
    switch_to_user_arm_v7m(user_stack, process_regs)
}

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn switch_to_user(
    _user_stack: *const u8,
    _process_regs: &mut [usize; 8],
) -> *const usize {
    unimplemented!()
}
