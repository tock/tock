//! Shared implementations for ARM Cortex-M4 MCUs.

#![crate_name = "cortexm4"]
#![crate_type = "rlib"]
#![feature(llvm_asm, naked_functions)]
#![no_std]

pub mod mpu;

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m4.
pub use cortexm::support;

pub use cortexm::nvic;
pub use cortexm::print_cortexm_state as print_cortexm4_state;
pub use cortexm::scb;
pub use cortexm::syscall;
pub use cortexm::systick;

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

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn systick_handler() {
    unimplemented!()
}

/// The `systick_handler` is called when the systick interrupt occurs, signaling
/// that an application executed for longer than its timeslice. This interrupt
/// handler is no longer responsible for signaling to the kernel thread that
/// an interrupt has occurred, but is slightly more efficient than the generic_isr
/// handler on account of not needing to mark the interrupt as pending.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn systick_handler() {
    llvm_asm!(
        "
    // Set thread mode to privileged to switch back to kernel mode.
    mov r0, #0
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    movw LR, #0xFFF9
    movt LR, #0xFFFF

    // This will resume in the switch to user function where application state
    // is saved and the scheduler can choose what to do next.
    "
    : : : : : "volatile" );
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn generic_isr() {
    unimplemented!()
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
    llvm_asm!(
        "
    // Set thread mode to privileged to ensure we are executing as the kernel.
    // This may be redundant if the interrupt happened while the kernel code
    // was executing.
    mov r0, #0
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    // This is a special address to return Thread mode with Main stack
    movw LR, #0xFFF9
    movt LR, #0xFFFF

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

    // r0 = 1 << (r0 & 31)
    movs r3, #1        // r3 = 1
    and r0, r0, #31    // r0 = r0 & 31
    lsl r0, r3, r0     // r0 = r3 << r0

    // Load the ICER register address.
    mov r3, #0xe180    // r3 = &NVIC.ICER
    movt r3, #0xe000

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
     * */
    /* r3 = &NVIC.ISPR */
    mov r3, #0xe200
    movt r3, #0xe000
    /* Set pending bit */
    str r0, [r3, r2, lsl #2]

    // Now we can return from the interrupt context and resume what we were
    // doing. If an app was executing we will switch to the kernel so it can
    // choose whether to service the interrupt.
    "
    : : : : "volatile" );
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn unhandled_interrupt() {
    unimplemented!()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub unsafe extern "C" fn unhandled_interrupt() {
    let mut interrupt_number: u32;

    // IPSR[8:0] holds the currently active interrupt
    llvm_asm!(
    "mrs    r0, ipsr                    "
    : "={r0}"(interrupt_number)
    :
    : "r0"
    :
    );

    interrupt_number = interrupt_number & 0x1ff;

    panic!("Unhandled Interrupt. ISR {} is active.", interrupt_number);
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn svc_handler() {
    unimplemented!()
}

/// This is called after a `svc` instruction, both when switching to userspace
/// and when userspace makes a system call.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn svc_handler() {
    llvm_asm!(
        "
    // First check to see which direction we are going in. If the link register
    // is something other than 0xfffffff9, then we are coming from an app which
    // has called a syscall.
    cmp lr, #0xfffffff9
    bne to_kernel

    // If we get here, then this is a context switch from the kernel to the
    // application. Set thread mode to unprivileged to run the application.
    mov r0, #1
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    // This is a special address to return Thread mode with Process stack
    movw lr, #0xfffd
    movt lr, #0xffff
    // Switch to the app.
    bx lr

  to_kernel:
    // An application called a syscall. We mark this in the global variable
    // `SYSCALL_FIRED` which is stored in the syscall file.
    // `UserspaceKernelBoundary` will use this variable to decide why the app
    // stopped executing.
    ldr r0, =SYSCALL_FIRED
    mov r1, #1
    str r1, [r0, #0]

    // Set thread mode to privileged as we switch back to the kernel.
    mov r0, #0
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    // This is a special address to return Thread mode with Main stack
    movw LR, #0xFFF9
    movt LR, #0xFFFF
    bx lr"
    : : : : "volatile" );
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn switch_to_user(
    _user_stack: *const u8,
    _process_regs: &mut [usize; 8],
) -> *const usize {
    unimplemented!()
}

/// Assembly function called from `UserspaceKernelBoundary` to switch to an
/// an application. This handles storing and restoring application state before
/// and after the switch.
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn switch_to_user(
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

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[inline(never)]
unsafe fn kernel_hardfault(faulting_stack: *mut u32) {
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
         \tprs 0x{:x} [ N {} Z {} C {} V {} Q {} GE {}{}{}{} ; ICI.IT {} T {} ; Exc {}-{} ]\r\n\
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
        (_estack as *const ()) as u32,
        (&_sstack as *const u32) as u32,
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

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn hard_fault_handler() {
    unimplemented!()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn hard_fault_handler() {
    let faulting_stack: *mut u32;
    let kernel_stack: bool;

    // First need to determine if this a kernel fault or a userspace fault.
    llvm_asm!(
    "
    mov    r1, 0     /* r1 = 0 */
    tst    lr, #4    /* bitwise AND link register to 0b100 */
    itte   eq        /* if lr==4, run next two instructions, else, run 3rd instruction. */
    mrseq  r0, msp   /* r0 = kernel stack pointer */
    addeq  r1, 1     /* r1 = 1, kernel was executing */
    mrsne  r0, psp   /* r0 = userland stack pointer */"
    : "={r0}"(faulting_stack), "={r1}"(kernel_stack)
    :
    : "r0", "r1"
    : "volatile" );

    if kernel_stack {
        // Need to determine if we had a stack overflow before we push anything
        // on to the stack. We check this by looking at the BusFault Status
        // Register's (BFSR) `LSPERR` and `STKERR` bits to see if the hardware
        // had any trouble stacking important registers to the stack during the
        // fault. If so, then we cannot use this stack while handling this fault
        // or we will trigger another fault.
        let stack_overflow: bool;
        llvm_asm!(
        "
        ldr   r2, =0xE000ED29  /* SCB BFSR register address */
        ldrb  r2, [r2]         /* r2 = BFSR */
        tst   r2, #0x30        /* r2 = BFSR & 0b00110000; LSPERR & STKERR bits */
        ite   ne               /* check if the result of that bitwise AND was not 0 */
        movne r3, #1           /* BFSR & 0b00110000 != 0; r3 = 1 */
        moveq r3, #0           /* BFSR & 0b00110000 == 0; r3 = 0 */"
        : "={r3}"(stack_overflow)
        :
        : "r3"
        : "volatile" );

        if stack_overflow {
            // The hardware couldn't use the stack, so we have no saved data and
            // we cannot use the kernel stack as is. We just want to report that
            // the kernel's stack overflowed, since that is essential for
            // debugging.
            //
            // To make room for a panic!() handler stack, we just re-use the
            // kernel's original stack. This should in theory leave the bottom
            // of the stack where the problem occurred untouched should one want
            // to further debug.
            llvm_asm!(
            "
            mov sp, r0   /* Set the stack pointer to _estack */"
            :
            : "{r0}"((_estack as *const ()) as u32)
            : : "volatile" );

            // Panic to show the correct error.
            panic!("kernel stack overflow");
        } else {
            // Show the normal kernel hardfault message.
            kernel_hardfault(faulting_stack);
        }
    } else {
        // Hard fault occurred in an app, not the kernel. The app should be
        // marked as in an error state and handled by the kernel.
        llvm_asm!(
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
        mov r1, #1              /* r1 = 1 */
        str r1, [r0, #0]        /* APP_HARD_FAULT = 1 */

        /* Set thread mode to privileged */
        mov r0, #0
        msr CONTROL, r0
        /* CONTROL writes must be followed by ISB */
        /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
        isb

        movw LR, #0xFFF9
        movt LR, #0xFFFF"
        : : : : "volatile" );
    }
}

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
