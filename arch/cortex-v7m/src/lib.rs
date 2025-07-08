// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Generic support for all Cortex-M platforms.

#![no_std]

// These constants are defined in the linker script.
extern "C" {
    static _estack: u8;
    static _sstack: u8;
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
extern "C" {
    /// ARMv7-M systick handler function.
    ///
    /// For documentation of this function, please see
    /// `CortexMVariant::SYSTICK_HANDLER`.
    pub fn systick_handler_arm_v7m();
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
core::arch::global_asm!(
    "
    .section .systick_handler_arm_v7m, \"ax\"
    .global systick_handler_arm_v7m
    .thumb_func
  systick_handler_arm_v7m:
    // Use the CONTROL register to set the thread mode to privileged to switch
    // back to kernel mode.
    //
    // CONTROL[1]: Stack status
    //   0 = Default stack (MSP) is used
    //   1 = Alternate stack is used
    // CONTROL[0]: Mode
    //   0 = Privileged in thread mode
    //   1 = User state in thread mode
    mov r0, #0                        // r0 = 0
    msr CONTROL, r0                   // CONTROL = 0
    // CONTROL writes must be followed by an Instruction Synchronization Barrier
    // (ISB). https://developer.arm.com/documentation/dai0321/latest
    isb                               // synchronization barrier

    // The link register is set to the `EXC_RETURN` value on exception entry. To
    // ensure we continue executing in the kernel we ensure the SPSEL bit is set
    // to 0 to use the main (kernel) stack.
    bfc lr, #2, #1                    // LR = LR & !(0x1<<2)

    // This will resume in the switch_to_user function where application state
    // is saved and the scheduler can choose what to do next.
    bx lr
    "
);

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
extern "C" {
    /// Handler of `svc` instructions on ARMv7-M.
    ///
    /// For documentation of this function, please see
    /// `CortexMVariant::SVC_HANDLER`.
    pub fn svc_handler_arm_v7m();
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
core::arch::global_asm!(
    "
    .section .svc_handler_arm_v7m, \"ax\"
    .global svc_handler_arm_v7m
    .thumb_func
  svc_handler_arm_v7m:
    // First check to see which direction we are going in. If the link register
    // (containing EXC_RETURN) has a 1 in the SPSEL bit (meaning the
    // alternative/process stack was in use) then we are coming from a process
    // which has called a syscall.
    ubfx r0, lr, #2, #1               // r0 = (LR & (0x1<<2)) >> 2
    cmp r0, #0                        // r0 (SPSEL bit) =≟ 0
    bne 100f // to_kernel             // if SPSEL == 1, jump to to_kernel

    // If we get here, then this is a context switch from the kernel to the
    // application. Use the CONTROL register to set the thread mode to
    // unprivileged to run the application.
    //
    // CONTROL[1]: Stack status
    //   0 = Default stack (MSP) is used
    //   1 = Alternate stack is used
    // CONTROL[0]: Mode
    //   0 = Privileged in thread mode
    //   1 = User state in thread mode
    mov r0, #1                        // r0 = 1
    msr CONTROL, r0                   // CONTROL = 1
    // CONTROL writes must be followed by an Instruction Synchronization Barrier
    // (ISB). https://developer.arm.com/documentation/dai0321/latest
    isb

    // The link register is set to the `EXC_RETURN` value on exception entry. To
    // ensure we execute using the process stack we set the SPSEL bit to 1
    // to use the alternate (process) stack.
    orr lr, lr, #4                    // LR = LR | 0b100

    // Switch to the app.
    bx lr

  100: // to_kernel
    // An application called a syscall. We mark this in the global variable
    // `SYSCALL_FIRED` which is stored in the syscall file.
    // `UserspaceKernelBoundary` will use this variable to decide why the app
    // stopped executing.
    ldr r0, =SYSCALL_FIRED            // r0 = &SYSCALL_FIRED
    mov r1, #1                        // r1 = 1
    str r1, [r0]                      // *SYSCALL_FIRED = 1

    // Use the CONTROL register to set the thread mode to privileged to switch
    // back to kernel mode.
    //
    // CONTROL[1]: Stack status
    //   0 = Default stack (MSP) is used
    //   1 = Alternate stack is used
    // CONTROL[0]: Mode
    //   0 = Privileged in thread mode
    //   1 = User state in thread mode
    mov r0, #0                        // r0 = 0
    msr CONTROL, r0                   // CONTROL = 0
    // CONTROL writes must be followed by an Instruction Synchronization Barrier
    // (ISB). https://developer.arm.com/documentation/dai0321/latest
    isb

    // The link register is set to the `EXC_RETURN` value on exception entry. To
    // ensure we continue executing in the kernel we ensure the SPSEL bit is set
    // to 0 to use the main (kernel) stack.
    bfc lr, #2, #1                    // LR = LR & !(0x1<<2)

    // Return to the kernel.
    bx lr
    "
);

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
extern "C" {
    /// Generic interrupt handler for ARMv7-M instruction sets.
    ///
    /// For documentation of this function, see `CortexMVariant::GENERIC_ISR`.
    pub fn generic_isr_arm_v7m();
}
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
core::arch::global_asm!(
        "
    .section .generic_isr_arm_v7m, \"ax\"
    .global generic_isr_arm_v7m
    .thumb_func
  generic_isr_arm_v7m:
    // Use the CONTROL register to set the thread mode to privileged to ensure
    // we are executing as the kernel. This may be redundant if the interrupt
    // happened while the kernel code was executing.
    //
    // CONTROL[1]: Stack status
    //   0 = Default stack (MSP) is used
    //   1 = Alternate stack is used
    // CONTROL[0]: Mode
    //   0 = Privileged in thread mode
    //   1 = User state in thread mode
    mov r0, #0                        // r0 = 0
    msr CONTROL, r0                   // CONTROL = 0
    // CONTROL writes must be followed by an Instruction Synchronization Barrier
    // (ISB). https://developer.arm.com/documentation/dai0321/latest
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
    mrs r0, IPSR                      // r0 = Interrupt Program Status Register (IPSR)
    and r0, #0xff                     // r0 = r0 & 0xFF; Get lowest 8 bits
    sub r0, #16                       // r0 = r0 - 16;   ISRs start at 16, so subtract 16 to get zero-indexed.

    // Now disable that interrupt in the NVIC.
    // High level:
    //    r0 = index
    //    NVIC.ICER[r0 / 32] = 1 << (r0 & 31)
    lsrs r2, r0, #5                   // r2 = r0 / 32
    // r0 = 1 << (r0 & 31)
    movs r3, #1                       // r3 = 1
    and r0, r0, #31                   // r0 = r0 & 31
    lsl r0, r3, r0                    // r0 = r3 << r0

    // Load the ICER register address.
    ldr r3, =0xe000e180               // r3 = &NVIC.ICER

    // Here:
    // - `r2` is index / 32
    // - `r3` is &NVIC.ICER
    // - `r0` is 1 << (index & 31)
    str r0, [r3, r2, lsl #2]          // *(r3 + r2 * 4) = r0

    // The pending bit in ISPR might be reset by hardware for pulse interrupts
    // at this point. So set it here again so the interrupt does not get lost in
    // `service_pending_interrupts()`.
    ldr r3, =0xe000e200               // r3 = &NVIC.ISPR
    str r0, [r3, r2, lsl #2]          // *(r3 + r2 * 4) = r0

    // The link register is set to the `EXC_RETURN` value on exception entry. To
    // ensure we continue executing in the kernel we ensure the SPSEL bit is set
    // to 0 to use the main (kernel) stack.
    bfc lr, #2, #1                    // LR = LR & !(0x1<<2)

    // Now we can return from the interrupt context and resume what we were
    // doing. If an app was executing we will switch to the kernel so it can
    // choose whether to service the interrupt.
    bx lr
    ");

/// Assembly function to switch into userspace and store/restore application
/// state.
///
/// For documentation of this function, please see
/// `CortexMVariant::switch_to_user`.
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
pub unsafe fn switch_to_user_arm_v7m(
    mut user_stack: *const usize,
    process_regs: &mut [usize; 8],
) -> *const usize {
    use core::arch::asm;
    asm!(
    "
    // Rust `asm!()` macro (as of May 2021) will not let us mark r6, r7 and r9
    // as clobbers. r6 and r9 is used internally by LLVM, and r7 is used for
    // the frame pointer. However, in the process of restoring and saving the
    // process's registers, we do in fact clobber r6, r7 and r9. So, we work
    // around this by doing our own manual saving of r6 using r2, r7 using r3,
    // r9 using r12, and then mark those as clobbered.
    mov r2, r6                        // r2 = r6
    mov r3, r7                        // r3 = r7
    mov r12, r9                       // r12 = r9

    // The arguments passed in are:
    // - `r0` is the bottom of the user stack
    // - `r1` is a reference to `CortexMStoredState.regs`

    // Load bottom of stack into Process Stack Pointer.
    msr psp, r0                       // PSP = r0

    // Load non-hardware-stacked registers from the process stored state. Ensure
    // that the address register (right now r1) is stored in a callee saved
    // register.
    ldmia r1, {{r4-r11}}              // r4 = r1[0], r5 = r1[1], ...

    // Generate a SVC exception to handle the context switch from kernel to
    // userspace. It doesn't matter which SVC number we use here as it is not
    // used in the exception handler. Data being returned from a syscall is
    // transferred on the app's stack.
    svc 0xff

    // When execution returns here we have switched back to the kernel from the
    // application.

    // Push non-hardware-stacked registers into the saved state for the
    // application.
    stmia r1, {{r4-r11}}              // r1[0] = r4, r1[1] = r5, ...

    // Update the user stack pointer with the current value after the
    // application has executed.
    mrs r0, PSP                       // r0 = PSP

    // Need to restore r6, r7 and r12 since we clobbered them when switching to
    // and from the app.
    mov r6, r2                        // r6 = r2
    mov r7, r3                        // r7 = r3
    mov r9, r12                       // r9 = r12
    ",
    inout("r0") user_stack,
    in("r1") process_regs,
    out("r2") _, out("r3") _, out("r4") _, out("r5") _, out("r8") _, out("r10") _,
    out("r11") _, out("r12") _);

    user_stack
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
/// Continue the hardfault handler for all hard-faults that occurred
/// during kernel execution. This function must never return.
unsafe extern "C" fn hard_fault_handler_arm_v7m_kernel(
    faulting_stack: *mut u32,
    stack_overflow: u32,
) -> ! {
    if stack_overflow != 0 {
        // Panic to show the correct error.
        panic!("kernel stack overflow");
    } else {
        // Show the normal kernel hardfault message.
        let stacked_r0: u32 = *faulting_stack.offset(0);
        let stacked_r1: u32 = *faulting_stack.offset(1);
        let stacked_r2: u32 = *faulting_stack.offset(2);
        let stacked_r3: u32 = *faulting_stack.offset(3);
        let stacked_r12: u32 = *faulting_stack.offset(4);
        let stacked_lr: u32 = *faulting_stack.offset(5);
        let stacked_pc: u32 = *faulting_stack.offset(6);
        let stacked_xpsr: u32 = *faulting_stack.offset(7);

        let mode_str = "Kernel";

        let shcsr: u32 = core::ptr::read_volatile(0xE000ED24 as *const u32);
        let cfsr: u32 = core::ptr::read_volatile(0xE000ED28 as *const u32);
        let hfsr: u32 = core::ptr::read_volatile(0xE000ED2C as *const u32);
        let mmfar: u32 = core::ptr::read_volatile(0xE000ED34 as *const u32);
        let bfar: u32 = core::ptr::read_volatile(0xE000ED38 as *const u32);

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

        let ici_it = (((stacked_xpsr >> 25) & 0x3) << 6) | ((stacked_xpsr >> 10) & 0x3f);
        let thumb_bit = ((stacked_xpsr >> 24) & 0x1) == 1;
        let exception_number = (stacked_xpsr & 0x1ff) as usize;

        panic!(
            "{} HardFault.\r\n\
         \tKernel version {}\r\n\
         \tr0  0x{:x}\r\n\
         \tr1  0x{:x}\r\n\
         \tr2  0x{:x}\r\n\
         \tr3  0x{:x}\r\n\
         \tr12 0x{:x}\r\n\
         \tlr  0x{:x}\r\n\
         \tpc  0x{:x}\r\n\
         \tpsr 0x{:x} [ N {} Z {} C {} V {} Q {} GE {}{}{}{} ; ICI.IT {} T {} ; Exc {}-{} ]\r\n\
         \tsp  0x{:x}\r\n\
         \ttop of stack     0x{:x}\r\n\
         \tbottom of stack  0x{:x}\r\n\
         \tSHCSR 0x{:x}\r\n\
         \tCFSR  0x{:x}\r\n\
         \tHSFR  0x{:x}\r\n\
         \tInstruction Access Violation:       {}\r\n\
         \tData Access Violation:              {}\r\n\
         \tMemory Management Unstacking Fault: {}\r\n\
         \tMemory Management Stacking Fault:   {}\r\n\
         \tMemory Management Lazy FP Fault:    {}\r\n\
         \tInstruction Bus Error:              {}\r\n\
         \tPrecise Data Bus Error:             {}\r\n\
         \tImprecise Data Bus Error:           {}\r\n\
         \tBus Unstacking Fault:               {}\r\n\
         \tBus Stacking Fault:                 {}\r\n\
         \tBus Lazy FP Fault:                  {}\r\n\
         \tUndefined Instruction Usage Fault:  {}\r\n\
         \tInvalid State Usage Fault:          {}\r\n\
         \tInvalid PC Load Usage Fault:        {}\r\n\
         \tNo Coprocessor Usage Fault:         {}\r\n\
         \tUnaligned Access Usage Fault:       {}\r\n\
         \tDivide By Zero:                     {}\r\n\
         \tBus Fault on Vector Table Read:     {}\r\n\
         \tForced Hard Fault:                  {}\r\n\
         \tFaulting Memory Address: (valid: {}) {:#010X}\r\n\
         \tBus Fault Address:       (valid: {}) {:#010X}\r\n\
         ",
            mode_str,
            option_env!("TOCK_KERNEL_VERSION").unwrap_or("unknown"),
            stacked_r0,
            stacked_r1,
            stacked_r2,
            stacked_r3,
            stacked_r12,
            stacked_lr,
            stacked_pc,
            stacked_xpsr,
            (stacked_xpsr >> 31) & 0x1,
            (stacked_xpsr >> 30) & 0x1,
            (stacked_xpsr >> 29) & 0x1,
            (stacked_xpsr >> 28) & 0x1,
            (stacked_xpsr >> 27) & 0x1,
            (stacked_xpsr >> 19) & 0x1,
            (stacked_xpsr >> 18) & 0x1,
            (stacked_xpsr >> 17) & 0x1,
            (stacked_xpsr >> 16) & 0x1,
            ici_it,
            thumb_bit,
            exception_number,
            ipsr_isr_number_to_str(exception_number),
            faulting_stack as u32,
            core::ptr::addr_of!(_estack) as u32,
            core::ptr::addr_of!(_sstack) as u32,
            shcsr,
            cfsr,
            hfsr,
            iaccviol,
            daccviol,
            munstkerr,
            mstkerr,
            mlsperr,
            ibuserr,
            preciserr,
            impreciserr,
            unstkerr,
            stkerr,
            lsperr,
            undefinstr,
            invstate,
            invpc,
            nocp,
            unaligned,
            divbysero,
            vecttbl,
            forced,
            mmfarvalid,
            mmfar,
            bfarvalid,
            bfar
        );
    }
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
extern "C" {
    /// ARMv7-M hardfault handler.
    ///
    /// For documentation of this function, please see
    /// `CortexMVariant::HARD_FAULT_HANDLER_HANDLER`.
    pub fn hard_fault_handler_arm_v7m();
}

#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
// First need to determine if this a kernel fault or a userspace fault, and store
// the unmodified stack pointer. Place these values in registers, then call
// a non-naked function, to allow for use of rust code alongside inline asm.
// Because calling a function increases the stack pointer, we have to check for a kernel
// stack overflow and adjust the stack pointer before we branch
core::arch::global_asm!(
    "
        .section .hard_fault_handler_arm_v7m, \"ax\"
        .global hard_fault_handler_arm_v7m
        .thumb_func
    hard_fault_handler_arm_v7m:
        mov    r2, 0     // r2 = 0
        tst    lr, #4    // bitwise AND link register to 0b100
        itte   eq        // if lr==4, run next two instructions, else, run 3rd instruction.
        mrseq  r0, msp   // r0 = kernel stack pointer
        addeq  r2, 1     // r2 = 1, kernel was executing
        mrsne  r0, psp   // r0 = userland stack pointer
        // Need to determine if we had a stack overflow before we push anything
        // on to the stack. We check this by looking at the BusFault Status
        // Register's (BFSR) `LSPERR` and `STKERR` bits to see if the hardware
        // had any trouble stacking important registers to the stack during the
        // fault. If so, then we cannot use this stack while handling this fault
        // or we will trigger another fault.
        ldr   r3, =0xE000ED29  // SCB BFSR register address
        ldrb  r3, [r3]         // r3 = BFSR
        tst   r3, #0x30        // r3 = BFSR & 0b00110000; LSPERR & STKERR bits
        ite   ne               // check if the result of that bitwise AND was not 0
        movne r1, #1           // BFSR & 0b00110000 != 0; r1 = 1
        moveq r1, #0           // BFSR & 0b00110000 == 0; r1 = 0
        and r5, r2, r1         // bitwise and r1 and r2, store in r5
        cmp  r5, #1            //  update condition codes to reflect if r1 == 1 && r2 == 1
        itt  eq                // if r5==1 run the next 2 instructions, else skip to branch
        // if true, The hardware couldn't use the stack, so we have no saved data and
        // we cannot use the kernel stack as is. We just want to report that
        // the kernel's stack overflowed, since that is essential for
        // debugging.
        //
        // To make room for a panic!() handler stack, we just re-use the
        // kernel's original stack. This should in theory leave the bottom
        // of the stack where the problem occurred untouched should one want
        // to further debug.
        ldreq  r4, ={estack} // load _estack into r4
        moveq  sp, r4        // Set the stack pointer to _estack
        // finally, if the fault occurred in privileged mode (r2 == 1), branch
        // to non-naked handler.
        cmp r2, #0
        // Per ARM calling convention, faulting stack is passed in r0, whether
        // there was a stack overflow in r1. This function must never return.
        bne {kernel_hard_fault_handler} // branch to kernel hard fault handler
        // Otherwise, the hard fault occurred in userspace. In this case, read
        // the relevant SCB registers:
        ldr r0, =SCB_REGISTERS    // Global variable address
        ldr r1, =0xE000ED14       // SCB CCR register address
        ldr r2, [r1, #0]          // CCR
        str r2, [r0, #0]
        ldr r2, [r1, #20]         // CFSR
        str r2, [r0, #4]
        ldr r2, [r1, #24]         // HFSR
        str r2, [r0, #8]
        ldr r2, [r1, #32]         // MMFAR
        str r2, [r0, #12]
        ldr r2, [r1, #36]         // BFAR
        str r2, [r0, #16]

        ldr r0, =APP_HARD_FAULT  // Global variable address
        mov r1, #1               // r1 = 1
        str r1, [r0, #0]         // APP_HARD_FAULT = 1

        // Set thread mode to privileged
        mov r0, #0
        msr CONTROL, r0
        // CONTROL writes must be followed by ISB
        // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html
        isb

        // The link register is set to the `EXC_RETURN` value on exception
        // entry. To ensure we continue executing in the kernel we ensure the
        // SPSEL bit is set to 0 to use the main (kernel) stack.
        bfc lr, #2, #1                    // LR = LR & !(0x1<<2)

        bx lr",
    estack = sym _estack,
    kernel_hard_fault_handler = sym hard_fault_handler_arm_v7m_kernel,
);

// Table 2.5
// http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0553a/CHDBIBGJ.html
pub fn ipsr_isr_number_to_str(isr_number: usize) -> &'static str {
    match isr_number {
        0 => "Thread Mode",
        1 => "Reserved",
        2 => "NMI",
        3 => "HardFault",
        4 => "MemManage",
        5 => "BusFault",
        6 => "UsageFault",
        7..=10 => "Reserved",
        11 => "SVCall",
        12 => "Reserved for Debug",
        13 => "Reserved",
        14 => "PendSV",
        15 => "SysTick",
        16..=255 => "IRQn",
        _ => "(Unknown! Illegal value?)",
    }
}

///////////////////////////////////////////////////////////////////
// Mock implementations for running tests on CI.
//
// Since tests run on the local architecture, we have to remove any
// ARM assembly since it will not compile.
///////////////////////////////////////////////////////////////////

#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe extern "C" fn systick_handler_arm_v7m() {
    unimplemented!()
}

#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe extern "C" fn svc_handler_arm_v7m() {
    unimplemented!()
}

#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe extern "C" fn generic_isr_arm_v7m() {
    unimplemented!()
}

#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe extern "C" fn switch_to_user_arm_v7m(
    _user_stack: *const u8,
    _process_regs: &mut [usize; 8],
) -> *const usize {
    unimplemented!()
}

#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
pub unsafe extern "C" fn hard_fault_handler_arm_v7m() {
    unimplemented!()
}
