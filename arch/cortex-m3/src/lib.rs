//! Shared implementations for ARM Cortex-M3 MCUs.

#![crate_name = "cortexm3"]
#![crate_type = "rlib"]
#![feature(llvm_asm, naked_functions)]
#![no_std]

pub mod mpu;

// Re-export the base generic cortex-m functions here as they are
// valid on cortex-m3.
pub use cortexm::support;

pub use cortexm::nvic;
pub use cortexm::print_cortexm_state as print_cortexm3_state;
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

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn systick_handler() {
    llvm_asm!(
        "
    /* Mark that the systick handler was called meaning that the process */
    /* stopped executing because it has exceeded its timeslice. */
    ldr r0, =SYSTICK_EXPIRED
    mov r1, #1
    str r1, [r0, #0]

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

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn generic_isr() {
    unimplemented!()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
/// All ISRs are caught by this handler which disables the NVIC and switches to the kernel.
pub unsafe extern "C" fn generic_isr() {
    llvm_asm!(
        "
    /* Skip saving process state if not coming from user-space */
    cmp lr, #0xfffffffd
    bne _ggeneric_isr_no_stacking

    /* We need the most recent kernel's version of r1, which points */
    /* to the Process struct's stored registers field. The kernel's r1 */
    /* lives in the second word of the hardware stacked registers on MSP */
    mov r1, sp
    ldr r1, [r1, #4]
    stmia r1, {r4-r11}

    /* Set thread mode to privileged */
    mov r0, #0
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    movw LR, #0xFFF9
    movt LR, #0xFFFF
  _ggeneric_isr_no_stacking:
    /* Find the ISR number by looking at the low byte of the IPSR registers */
    mrs r0, IPSR
    and r0, #0xff
    /* ISRs start at 16, so substract 16 to get zero-indexed */
    sub r0, #16

    /*
     * High level:
     *    NVIC.ICER[r0 / 32] = 1 << (r0 & 31)
     * */
    lsrs r2, r0, #5 /* r2 = r0 / 32 */

    /* r0 = 1 << (r0 & 31) */
    movs r3, #1        /* r3 = 1 */
    and r0, r0, #31    /* r0 = r0 & 31 */
    lsl r0, r3, r0     /* r0 = r3 << r0 */

    /* r3 = &NVIC.ICER */
    mov r3, #0xe180
    movt r3, #0xe000

    /* here:
     *
     *  `r2` is r0 / 32
     *  `r3` is &NVIC.ICER
     *  `r0` is 1 << (r0 & 31)
     *
     * So we just do:
     *
     *  `*(r3 + r2 * 4) = r0`
     *
     *  */
    str r0, [r3, r2, lsl #2]

    /* The pending bit in ISPR might be reset by hardware for pulse interrupts
     * at this point. So set it here again so the interrupt does not get lost
     * in service_pending_interrupts()
     * */
    /* r3 = &NVIC.ISPR */
    mov r3, #0xe200
    movt r3, #0xe000
    /* Set pending bit */
    str r0, [r3, r2, lsl #2]"
    : : : : "volatile" );
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn svc_handler() {
    unimplemented!()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[naked]
pub unsafe extern "C" fn svc_handler() {
    llvm_asm!(
        "
    cmp lr, #0xfffffff9
    bne to_kernel

    /* Set thread mode to unprivileged */
    mov r0, #1
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

    movw lr, #0xfffd
    movt lr, #0xffff
    bx lr
  to_kernel:
    ldr r0, =SYSCALL_FIRED
    mov r1, #1
    str r1, [r0, #0]

    /* Set thread mode to privileged */
    mov r0, #0
    msr CONTROL, r0
    /* CONTROL writes must be followed by ISB */
    /* http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0321a/BIHFJCAC.html */
    isb

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

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[no_mangle]
/// r0 is top of user stack, r1 is reference to `CortexMStoredState.regs`
pub unsafe extern "C" fn switch_to_user(
    mut user_stack: *const usize,
    process_regs: &mut [usize; 8],
) -> *const usize {
    llvm_asm!("
    /* Load bottom of stack into Process Stack Pointer */
    msr psp, $0

    /* Load non-hardware-stacked registers from Process stack */
    /* Ensure that $2 is stored in a callee saved register */
    ldmia $2, {r4-r11}

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

    /* Push non-hardware-stacked registers into Process struct's */
    /* regs field */
    stmia $2, {r4-r11}


    mrs $0, PSP /* PSP into r0 */"
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
            : "volatile" );

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
