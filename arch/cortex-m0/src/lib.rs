// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Shared implementations for ARM Cortex-M0 MCUs.

#![no_std]

use core::fmt::Write;

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m0.
pub use cortexm::support;

pub use cortexm::nvic;
pub use cortexm::syscall;

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
struct HardFaultStackedRegisters {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
    lr: u32,
    pc: u32,
    xpsr: u32,
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
/// Handle a hard fault that occurred in the kernel. This function is invoked
/// by the naked hard_fault_handler function.
unsafe extern "C" fn hard_fault_handler_kernel(faulting_stack: *mut u32) -> ! {
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

    panic!(
        "Kernel HardFault.\r\n\
         \tKernel version {}\r\n\
         \tr0  0x{:x}\r\n\
         \tr1  0x{:x}\r\n\
         \tr2  0x{:x}\r\n\
         \tr3  0x{:x}\r\n\
         \tr12  0x{:x}\r\n\
         \tlr  0x{:x}\r\n\
         \tpc  0x{:x}\r\n\
         \txpsr  0x{:x}\r\n\
         ",
        option_env!("TOCK_KERNEL_VERSION").unwrap_or("unknown"),
        hardfault_stacked_registers.r0,
        hardfault_stacked_registers.r1,
        hardfault_stacked_registers.r2,
        hardfault_stacked_registers.r3,
        hardfault_stacked_registers.r12,
        hardfault_stacked_registers.lr,
        hardfault_stacked_registers.pc,
        hardfault_stacked_registers.xpsr
    );
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
unsafe extern "C" fn generic_isr() {
    unimplemented!()
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
extern "C" {
    /// All ISRs are caught by this handler which disables the NVIC and switches to the kernel.
    pub fn generic_isr();
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
core::arch::global_asm!(
    "
    .section .generic_isr, \"ax\"
    .global generic_isr
    .thumb_func
  generic_isr:
    /* Skip saving process state if not coming from user-space */
    ldr r0, 300f // MEXC_RETURN_PSP
    cmp lr, r0
    bne 100f

    /* We need to make sure the kernel cotinues the execution after this ISR */
    movs r0, #0
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* https://developer.arm.com/documentation/dui0662/b/The-Cortex-M0--Processor/Programmers-model/Core-registers */
    isb

    /* We need the most recent kernel's version of r1, which points */
    /* to the Process struct's stored registers field. The kernel's r1 */
    /* lives in the second word of the hardware stacked registers on MSP */
    mov r1, sp
    ldr r1, [r1, #4]
    str r4, [r1, #16]
    str r5, [r1, #20]
    str r6, [r1, #24]
    str r7, [r1, #28]

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

    ldr r0, 200f // MEXC_RETURN_MSP
    mov lr, r0
100: // _ggeneric_isr_no_stacking
    /* Find the ISR number by looking at the low byte of the IPSR registers */
    mrs r0, IPSR
    movs r1, #0xff
    ands r0, r1
    /* ISRs start at 16, so subtract 16 to get zero-indexed */
    subs r0, r0, #16

    /*
     * High level:
     *    NVIC.ICER[r0 / 32] = 1 << (r0 & 31)
     * */
    /* r3 = &NVIC.ICER[r0 / 32] */
    ldr r2, 101f      /* r2 = &NVIC.ICER */
    lsrs r3, r0, #5   /* r3 = r0 / 32 */
    lsls r3, r3, #2   /* ICER is word-sized, so multiply offset by 4 */
    adds r3, r3, r2   /* r3 = r2 + r3 */

    /* r2 = 1 << (r0 & 31) */
    movs r2, #31      /* r2 = 31 */
    ands r0, r2       /* r0 = r0 & r2 */
    subs r2, r2, #30  /* r2 = r2 - 30 i.e. r2 = 1 */
    lsls r2, r2, r0   /* r2 = 1 << r0 */

    /* *r3 = r2 */
    str r2, [r3]

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
101: // NVICICER
  .word 0xE000E180
200: // MEXC_RETURN_MSP
  .word 0xFFFFFFF9
300: // MEXC_RETURN_PSP
  .word 0xFFFFFFFD"
);

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
unsafe extern "C" fn systick_handler_m0() {
    unimplemented!()
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
extern "C" {
    /// The `systick_handler` is called when the systick interrupt occurs, signaling
    /// that an application executed for longer than its timeslice. This interrupt
    /// handler is no longer responsible for signaling to the kernel thread that an
    /// interrupt has occurred, but is slightly more efficient than the
    /// `generic_isr` handler on account of not needing to mark the interrupt as
    /// pending.
    pub fn systick_handler_m0();
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
core::arch::global_asm!(
    "
    .section .systick_handler_m0, \"ax\"
    .global systick_handler_m0
    .thumb_func
  systick_handler_m0:

    // Set thread mode to privileged to switch back to kernel mode.
    movs r0, #0
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    ldr r0, 100f // ST_EXC_RETURN_MSP

    // This will resume in the switch to user function where application state
    // is saved and the scheduler can choose what to do next.
    bx   r0
.align 4
100: // ST_EXC_RETURN_MSP
  .word 0xFFFFFFF9
    "
);

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
unsafe extern "C" fn svc_handler() {
    unimplemented!()
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
extern "C" {
    pub fn svc_handler();
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
core::arch::global_asm!(
    "
  .section .svc_handler, \"ax\"
  .global svc_handler
  .thumb_func
svc_handler:
  ldr r0, 200f // EXC_RETURN_MSP
  cmp lr, r0
  bne 100f
  movs r0, #1
  msr CONTROL, r0
  /* CONTROL writes must be followed by ISB */
    /* https://developer.arm.com/documentation/dui0662/b/The-Cortex-M0--Processor/Programmers-model/Core-registers */
  isb
  ldr r1, 300f // EXC_RETURN_PSP
  bx r1

100: // to_kernel
  movs r0, #0
  msr CONTROL, r0
  /* CONTROL writes must be followed by ISB */
    /* https://developer.arm.com/documentation/dui0662/b/The-Cortex-M0--Processor/Programmers-model/Core-registers */
  isb
  ldr r0, =SYSCALL_FIRED
  movs r1, #1
  str r1, [r0, #0]
  ldr r1, 200f
  bx r1

.align 4
200: // EXC_RETURN_MSP
  .word 0xFFFFFFF9
300: // EXC_RETURN_PSP
  .word 0xFFFFFFFD
  "
);

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
unsafe extern "C" fn hard_fault_handler() {
    unimplemented!()
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
extern "C" {
    pub fn hard_fault_handler();
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
// If `kernel_stack` is non-zero, then hard-fault occurred in
// kernel, otherwise the hard-fault occurred in user.
core::arch::global_asm!(
"
    .section .hard_fault_handler, \"ax\"
    .global hard_fault_handler
    .thumb_func
  hard_fault_handler:
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
    beq 100f

// _hardfault_psp:
    mrs r0, psp
    b 200f

100: // _hardfault_msp
    mrs r0, msp
    adds r1, #1

200: // _hardfault_exit

    // If the hard-fault occurred while executing the kernel (r1 != 0),
    // jump to the non-naked kernel hard fault handler. This handler
    // MUST NOT return. The faulting stack is passed as the first argument
    // (r0).
    cmp r1, #0                           // Check if app (r1==0) or kernel (r1==1) fault.
    beq 400f                             // If app fault, skip to app handling.
    ldr r2, ={kernel_hard_fault_handler} // Load address of fault handler.
    bx r2                                // Branch to the non-naked fault handler.

400: // _hardfault_app
    // Otherwise, store that a hardfault occurred in an app, store some CPU
    // state and finally return to the kernel stack:
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

    // Load the FEXC_RETURN_MSP LR address and return to it, to switch to the
    // kernel (MSP) stack:
    ldr r0, 300f
    mov lr, r0
    bx lr

    .align 4
300: // FEXC_RETURN_MSP
    .word 0xFFFFFFF9
    ",
    kernel_hard_fault_handler = sym hard_fault_handler_kernel,
);

// Enum with no variants to ensure that this type is not instantiable. It is
// only used to pass architecture-specific constants and functions via the
// `CortexMVariant` trait.
pub enum CortexM0 {}

impl cortexm::CortexMVariant for CortexM0 {
    const GENERIC_ISR: unsafe extern "C" fn() = generic_isr;
    const SYSTICK_HANDLER: unsafe extern "C" fn() = systick_handler_m0;
    const SVC_HANDLER: unsafe extern "C" fn() = svc_handler;
    const HARD_FAULT_HANDLER: unsafe extern "C" fn() = hard_fault_handler;

    // Mock implementation for tests on Travis-CI.
    #[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
    unsafe fn switch_to_user(
        _user_stack: *const usize,
        _process_regs: &mut [usize; 8],
    ) -> *const usize {
        unimplemented!()
    }

    #[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
    unsafe fn switch_to_user(
        mut user_stack: *const usize,
        process_regs: &mut [usize; 8],
    ) -> *const usize {
        use core::arch::asm;
        asm!("
    // Rust `asm!()` macro (as of May 2021) will not let us mark r6, r7 and r9
    // as clobbers. r6 and r9 is used internally by LLVM, and r7 is used for
    // the frame pointer. However, in the process of restoring and saving the
    // process's registers, we do in fact clobber r6, r7 and r9. So, we work
    // around this by doing our own manual saving of r6 using r2, r7 using r3,
    // r9 using r12, and then mark those as clobbered.
    mov r2, r6
    mov r3, r7
    mov r12, r9

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

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

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

    // Manually restore r6, r7 and r9.
    mov r6, r2
    mov r7, r3
    mov r9, r12

    ",
    inout("r0") user_stack,
    in("r1") process_regs,
    out("r2") _, out("r3") _, out("r4") _, out("r5") _, out("r8") _,
    out("r10") _, out("r11") _, out("r12") _);

        user_stack
    }

    #[inline]
    unsafe fn print_cortexm_state(writer: &mut dyn Write) {
        cortexm::print_cortexm_state(writer)
    }
}
