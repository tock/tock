//! Shared implementations for ARM Cortex-M0 MCUs.

#![crate_name = "cortexm0"]
#![crate_type = "rlib"]
#![feature(asm, naked_functions)]
#![no_std]

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m0.
pub use cortexm::support;

pub use cortexm::kernel_hardfault_arm_v7m;
pub use cortexm::nvic;
pub use cortexm::print_cortexm_state as print_cortexm0_state;
pub use cortexm::syscall;

extern "C" {
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();
    static mut _sstack: u32;
    static mut _szero: u32;
    static mut _ezero: u32;
    static mut _etext: u32;
    static mut _srelocate: u32;
    static mut _erelocate: u32;
}

/// The `systick_handler_armv6` is called when the systick interrupt occurs, signaling
/// that an application executed for longer than its timeslice. This interrupt
/// handler is no longer responsible for signaling to the kernel thread that an
/// interrupt has occurred, but is slightly more efficient than the
/// `generic_isr` handler on account of not needing to mark the interrupt as
/// pending.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn systick_handler() {
    asm!(
        "
    // Set thread mode to privileged to switch back to kernel mode.
    movs r0, #0
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    ldr r0, SYSTICK_MEXC_RETURN_MSP

    // This will resume in the switch to user function where application state
    // is saved and the scheduler can choose what to do next.
    bx   r0
    .align 4
    SYSTICK_MEXC_RETURN_MSP:
    .word 0xFFFFFFF9
    ",
        options(noreturn)
    );
}

/// This is called after a `svc` instruction, both when switching to userspace
/// and when userspace makes a system call.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
/// All ISRs are caught by this handler which disables the NVIC and switches to the kernel.
pub unsafe extern "C" fn generic_isr() {
    asm!(
        "
    // First check to see which direction we are going in. If the link register
    // is something other than 0xfffffff9, then we are coming from an app which
    // has called a syscall.
    ldr r0, SVC_MEXC_RETURN_PSP
    cmp lr, r0
    bne to_kernel

    // If we get here, then this is a context switch from the kernel to the
    // application. Set thread mode to unprivileged to run the application.
    movs r0, #1
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    // This is a special address to return Thread mode with Process stack
    ldr r0, SVC_MEXC_RETURN_PSP
    // Switch to the app.
    bx r0

  to_kernel:
    // An application called a syscall. We mark this in the global variable
    // `SYSCALL_FIRED` which is stored in the syscall file.
    // `UserspaceKernelBoundary` will use this variable to decide why the app
    // stopped executing.
    ldr r0, =SYSCALL_FIRED
    movs r1, #1
    str r1, [r0, #0]

    // Set thread mode to privileged as we switch back to the kernel.
    movs r0, #0
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    // This is a special address to return Thread mode with Main stack
    ldr r0, SVC_MEXC_RETURN_MSP
    bx r0
    .align 4
    SVC_MEXC_RETURN_MSP:
    .word 0xFFFFFFF9
    SVC_MEXC_RETURN_PSP:
    .word 0xFFFFFFFD",
        options(noreturn)
    );
}

/// All ISRs are caught by this handler. This must ensure the interrupt is
/// disabled (per Tock's interrupt model) and then as quickly as possible resume
/// the main thread (i.e. leave the interrupt context). The interrupt will be
/// marked as pending and handled when the scheduler checks if there are any
/// pending interrupts.
///
/// If the ISR is called while an app is running, this will switch control to
/// the kernel.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn generic_isr() {
    asm!(
        "
    // Set thread mode to privileged to ensure we are executing as the kernel.
    // This may be redundant if the interrupt happened while the kernel code
    // was executing.
    movs r0, #0
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    // Now need to disable the interrupt that fired in the NVIC to ensure it
    // does not trigger again before the scheduler has a chance to handle it. We
    // do this here in assembly for performance.
    //
    // The general idea is:
    // 1. Get the index of the interrupt that occurred.
    // 2. Set the disable bit for that interrupt in the NVIC.

    // Find the ISR number (`index`) by looking at the low byte of the IPSR
    // registers.
    mrs r0, IPSR       // r0 = Interrupt Program Status Register (IPSR)
    and r0, #0xff      // r0 = r0 & 0xFF
    sub r0, #16        // ISRs start at 16, so subtract 16 to get zero-indexed.

    // Now disable that interrupt in the NVIC.
    // High level:
    //    r0 = index
    //    NVIC.ICER[r0 / 32] = 1 << (r0 & 31)
    //
    lsrs r2, r0, #5    // r2 = r0 / 32

    push {{r4-r7}}
    mov  r4, r8
    mov  r5, r9
    mov  r6, r10
    mov  r7, r11
    str r4, [r1, #0]
    str r5, [r1, #4]
    str r6, [r1, #8]
    str r7, [r1, #12]
    pop {{r4-r7}}

    // Load the ICER register address.
    ldr r3, NVICICER

    // Here:
    // - `r2` is index / 32
    // - `r3` is &NVIC.ICER
    // - `r0` is 1 << (index & 31)
    //
    // So we just do:
    //
    //  `*(r3 + r2 * 4) = r0`
    //
    str r0, [r3, r2, lsl #2]

    /* The pending bit in ISPR might be reset by hardware for pulse interrupts
     * at this point. So set it here again so the interrupt does not get lost
     * in service_pending_interrupts()
     *
     * The NVIC.ISPR base is 0xE000E200, which is 0x20 (aka #32) above the
     * NVIC.ICER base.  Calculate the ISPR address by offsetting from the ICER
     * address so as to avoid re-doing the [r0 / 32] index math.
     */
    adds r3, #32
    str r2, [r3]

    bx lr /* return here since we have extra words in the assembly */

.align 4
NVICICER:
  .word 0xE000E180
MEXC_RETURN_MSP:
  .word 0xFFFFFFF9
MEXC_RETURN_PSP:
  .word 0xFFFFFFFD",
        options(noreturn)
    );
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn systick_handler() {
    unimplemented!()
}

/// The `systick_handler` is called when the systick interrupt occurs, signaling
/// that an application executed for longer than its timeslice. This interrupt
/// handler is no longer responsible for signaling to the kernel thread that an
/// interrupt has occurred, but is slightly more efficient than the
/// `generic_isr` handler on account of not needing to mark the interrupt as
/// pending.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn systick_handler() {
    asm!(
        "
    // Set thread mode to privileged to switch back to kernel mode.
    movs r0, #0
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    ldr r0, ST_EXC_RETURN_MSP

    // This will resume in the switch to user function where application state
    // is saved and the scheduler can choose what to do next.
    bx   r0
.align 4
ST_EXC_RETURN_MSP:
  .word 0xFFFFFFF9
    ",
        options(noreturn)
    );
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn svc_handler() {
    unimplemented!()
}

/// Assembly function called from `UserspaceKernelBoundary` to switch to an
/// an application. This handles storing and restoring application state before
/// and after the switch.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn svc_handler() {
    asm!(
        "
  ldr r0, EXC_RETURN_MSP
  cmp lr, r0
  bne to_kernel
  ldr r1, EXC_RETURN_PSP
  bx r1

to_kernel:
  ldr r0, =SYSCALL_FIRED
  movs r1, #1
  str r1, [r0, #0]
  ldr r1, EXC_RETURN_MSP
  bx r1

.align 4
EXC_RETURN_MSP:
  .word 0xFFFFFFF9
EXC_RETURN_PSP:
  .word 0xFFFFFFFD
  ",
        options(noreturn)
    );
}

    // Load bottom of stack into Process Stack Pointer.
    msr psp, $0

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn switch_to_user(
    mut user_stack: *const u8,
    process_regs: &mut [usize; 8],
) -> *mut u8 {
    asm!("
    // Manually save r6 in r2 and r7 in r3 since as of Feb 2021 asm!() will not
    // let us mark r6 or r7 as clobbers.
    mov r2, r6
    mov r3, r7

    /* Load non-hardware-stacked registers from Process stack */
    ldmia r1!, {{r4-r7}}
    mov r11, r7
    mov r10, r6
    mov r9,  r5
    mov r8,  r4
    ldmia r1!, {{r4-r7}}
    subs r1, 32 /* Restore pointer to process_regs
                /* ldmia! added a 32-byte offset */

    /* Load bottom of stack into Process Stack Pointer */
    msr psp, r0

    // When execution returns here we have switched back to the kernel from the
    // application.

    /* Store non-hardware-stacked registers in process_regs */
    /* r1 still points to process_regs because we are clobbering all */
    /* non-hardware-stacked registers */
    str r4, [r1, #16]
    str r5, [r1, #20]
    str r6, [r1, #24]
    str r7, [r1, #28]

    mov  r4, r8
    mov  r5, r9
    mov  r6, r10
    mov  r7, r11

    str r4, [r1, #0]
    str r5, [r1, #4]
    str r6, [r1, #8]
    str r7, [r1, #12]

    mrs r0, PSP /* PSP into user_stack */

    // Manually restore r6 and r7.
    mov r6, r2
    mov r7, r3

    ",
    inout("r0") user_stack,
    in("r1") process_regs,
    out("r2") _, out("r3") _, out("r4") _, out("r5") _, out("r8") _, out("r9") _,
    out("r10") _, out("r11") _);

    user_stack as *mut u8
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
/// Continue the hardfault handler. This function is not `#[naked]`, meaning we can mix
/// `asm!()` and Rust. We separate this logic to not have to write the entire fault
/// handler entirely in assembly.
unsafe extern "C" fn hard_fault_handler_continued(
    faulting_stack: *mut u32,
    kernel_stack: u32,
    stack_overflow: u32,
) {
    if kernel_stack != 0 {
        if stack_overflow != 0 {
            // Panic to show the correct error.
            panic!("kernel stack overflow");
        } else {
            // Show the normal kernel hardfault message.
            kernel_hardfault_arm_v7m(faulting_stack);
        }
    } else {
        // Hard fault occurred in an app, not the kernel. The app should be
        // marked as in an error state and handled by the kernel.
        asm!(
            "
        /* Read the relevant SCB registers. */
        ldr r0, =SCB_REGISTERS  /* Global variable address */
        ldr r1, =0xE000ED14     /* SCB CCR register address */
        ldr r2, [r1, #0]        /* CCR */
        str r2, [r0, #0]
        ldr r2, [r1, #20]       /* CFSR */
        str r2, [r0, #4]
        ldr r2, [r1, #24]       /* HFSR */
        str r2, [r0, #8]
        ldr r2, [r1, #32]       /* MMFAR */
        str r2, [r0, #12]
        ldr r2, [r1, #36]       /* BFAR */
        str r2, [r0, #16]

        ldr r0, =APP_HARD_FAULT /* Global variable address */
        movs r1, #1              /* r1 = 1 */
        str r1, [r0, #0]        /* APP_HARD_FAULT = 1 */

        /* Set thread mode to privileged */
        movs r0, #0
        msr CONTROL, r0
        /* CONTROL writes must be followed by ISB */
        /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
        isb

        ldr r0, HARD_FAULT_MEXC_RETURN_MSP
        mov lr, r0
        .align 4
        HARD_FAULT_MEXC_RETURN_MSP:
        .word 0xFFFFFFF9",
            out("r1") _,
            out("r0") _,
            out("r2") _,
            options(nostack),
        );
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[inline(never)]
unsafe fn kernel_hardfault(faulting_stack: *mut u32) {
    let hardfault_stacked_registers = HardFaultStackedRegisters {
        r0: *faulting_stack.offset(0),
        r1: *faulting_stack.offset(1),
        r2: *faulting_stack.offset(2),
        r3: *faulting_stack.offset(3),
        r12: *faulting_stack.offset(4),
        lr: *faulting_stack.offset(5),
        pc: *faulting_stack.offset(6),
        xpsr: *faulting_stack.offset(7),
    };

    // NOTE: Unlike Cortex-M3, `panic!` does not seem to work
    //       here. `panic!` seems to be producing wrong `PanicInfo`
    //       value. Therefore as a workaround, capture the stacked
    //       registers and invoke a breakpoint.
    //
    asm!(
        "
         bkpt
1:
         b 1b
         "
    );
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn generic_isr() {
    unimplemented!()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
/// Continue the hardfault handler. This function is not `#[naked]`, meaning we
/// can mix `asm!()` and Rust. We separate this logic to not have to write the
/// entire fault handler entirely in assembly.
unsafe extern "C" fn hard_fault_handler_continued(faulting_stack: *mut u32, kernel_stack: u32) {
    if kernel_stack != 0 {
        kernel_hardfault(faulting_stack);
    } else {
        // hard fault occurred in an app, not the kernel. The app should be
        // marked as in an error state and handled by the kernel
        asm!(
            "
            ldr r0, =APP_HARD_FAULT
            movs r1, #1 /* Fault */
            str r1, [r0, #0]

            /*
            * NOTE:
            * -----
            *
            * Even though ARMv6-M SCB and Control registers
            * are different from ARMv7-M, they are still compatible
            * with each other. So, we can keep the same code as
            * ARMv7-M.
            *
            * ARMv6-M however has no _privileged_ mode.
            */

            /* Read the SCB registers. */
            ldr r0, =SCB_REGISTERS
            ldr r1, =0xE000ED14
            ldr r2, [r1, #0] /* CCR */
            str r2, [r0, #0]
            ldr r2, [r1, #20] /* CFSR */
            str r2, [r0, #4]
            ldr r2, [r1, #24] /* HFSR */
            str r2, [r0, #8]
            ldr r2, [r1, #32] /* MMFAR */
            str r2, [r0, #12]
            ldr r2, [r1, #36] /* BFAR */
            str r2, [r0, #16]

            /* Set thread mode to privileged */
            movs r0, #0
            msr CONTROL, r0
            /* No ISB required on M0 */
            /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */

            ldr r0, FEXC_RETURN_MSP
            bx r0
    .align 4
    FEXC_RETURN_MSP:
      .word 0xFFFFFFF9
        "
        );
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn hard_fault_handler() {
    // If `kernel_stack` is non-zero, then hard-fault occurred in
    // kernel, otherwise the hard-fault occurred in user.
    asm!("
    /*
     * Will be incremented to 1 when we determine that it was a fault
     * in the kernel
     */
    movs r1, #0
    /*
     * r2 is used for testing and r3 is used to store lr
     */
    mov r3, lr

    movs r2, #4
    tst r3, r2
    beq _hardfault_msp

_hardfault_psp:
    mrs r0, psp
    b _hardfault_exit

_hardfault_msp:
    mrs r0, msp
    adds r1, #1

_hardfault_exit:

    b {}    // Branch to the non-naked fault handler.
    bx lr   // If continued function returns, we need to manually branch to
            // link register.
    ",
    sym hard_fault_handler_continued,
    options(noreturn));
}

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn systick_handler() {
    unimplemented!()
}

// #[cfg(all(target_arch = "arm", target_os = "none"))]
// #[naked]
// pub unsafe extern "C" fn hard_fault_handler() {
//     let faulting_stack: *mut u32;
//     let kernel_stack: bool;

//     // If `kernel_stack` is non-zero, then hard-fault occurred in
//     // kernel, otherwise the hard-fault occurrend in user.
//     llvm_asm!("
//     /*
//      * Will be incremented to 1 when we determine that it was a fault
//      * in the kernel
//      */
//     movs r1, #0
//     /*
//      * r2 is used for testing and r3 is used to store lr
//      */
//     mov r3, lr

//     movs r2, #4
//     tst r3, r2
//     beq _hardfault_msp

// _hardfault_psp:
//     mrs r0, psp
//     b _hardfault_exit

// _hardfault_msp:
//     mrs r0, msp
//     adds r1, #1

// _hardfault_exit:
//     "
//     : "={r0}"(faulting_stack), "={r1}"(kernel_stack)
//     :
//     : "r0", "r1", "r2", "r3"
//     : "volatile"
//     );

//     if kernel_stack {
//         kernel_hardfault(faulting_stack);
//     } else {
//         // hard fault occurred in an app, not the kernel. The app should be
//         // marked as in an error state and handled by the kernel
//         llvm_asm!("
//              ldr r0, =APP_HARD_FAULT
//              movs r1, #1 /* Fault */
//              str r1, [r0, #0]

//              /*
//               * NOTE:
//               * -----
//               *
//               * Even though ARMv6-M SCB and Control registers
//               * are different from ARMv7-M, they are still compatible
//               * with each other. So, we can keep the same code as
//               * ARMv7-M.
//               *
//               * ARMv6-M however has no _privileged_ mode.
//               */
//              /* Read the SCB registers. */
//              ldr r0, =SCB_REGISTERS
//              ldr r1, =0xE000ED14
//              ldr r2, [r1, #0] /* CCR */
//              str r2, [r0, #0]
//              ldr r2, [r1, #20] /* CFSR */
//              str r2, [r0, #4]
//              ldr r2, [r1, #24] /* HFSR */
//              str r2, [r0, #8]
//              ldr r2, [r1, #32] /* MMFAR */
//              str r2, [r0, #12]
//              ldr r2, [r1, #36] /* BFAR */
//              str r2, [r0, #16]

//              /* Set thread mode to privileged */
//              movs r0, #0
//              msr CONTROL, r0
//              /* No ISB required on M0 */
//              /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
//              ldr r0, FEXC_RETURN_MSP
//              bx r0
// .align 4
// FEXC_RETURN_MSP:
//   .word 0xFFFFFFF9
//              "
//              :
//              :
//              :
//              : "volatile");
//     }
// }
