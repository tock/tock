#![crate_name = "cortexm4"]
#![crate_type = "rlib"]
#![feature(asm,const_fn,naked_functions)]
#![no_std]

extern crate kernel;

pub mod mpu;
pub mod nvic;
pub mod systick;
pub mod scb;

#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn systick_handler() {}

#[cfg(target_os = "none")]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn systick_handler() {
    asm!("
        /* Skip saving process state if not coming from user-space */
        cmp lr, #0xfffffffd
        bne _systick_handler_no_stacking

        /* We need the most recent kernel's version of r0, which points */
        /* to the Process struct's stored registers field. The kernel's r0 */
        /* lives in the first word of the hardware stacked registers on MSP */
        mov r0, sp
        ldr r0, [r0, #0]

        /* Push non-hardware-stacked registers onto Process stack */
        /* r0 points to user stack (see to_kernel) */
        stmia r0, {r4-r11}
    _systick_handler_no_stacking:
        ldr r0, =OVERFLOW_FIRED
        mov r1, #1
        str r1, [r0, #0]

        /* Set thread mode to privileged */
        mov r0, #0
        msr CONTROL, r0

        movw LR, #0xFFF9
        movt LR, #0xFFFF
         ");
}


#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn generic_isr() {}

#[cfg(target_os = "none")]
#[no_mangle]
#[naked]
/// All ISRs are caught by this handler which indirects to a custom handler by
/// indexing into `INTERRUPT_TABLE` based on the ISR number.
pub unsafe extern "C" fn generic_isr() {
    asm!("
    /* Skip saving process state if not coming from user-space */
    cmp lr, #0xfffffffd
    bne _ggeneric_isr_no_stacking

    /* We need the most recent kernel's version of r0, which points */
    /* to the Process struct's stored registers field. The kernel's r0 */
    /* lives in the first word of the hardware stacked registers on MSP */
    mov r0, sp
    ldr r0, [r0, #0]

    /* Push non-hardware-stacked registers onto Process stack */
    /* r0 points to user stack (see to_kernel) */
    stmia r0, {r4-r11}

    /* Set thread mode to privileged */
    mov r0, #0
    msr CONTROL, r0

    movw LR, #0xFFF9
    movt LR, #0xFFFF
_ggeneric_isr_no_stacking:
    /* Find the ISR number by looking at the low byte of the IPSR registers */
    mrs r0, IPSR
    and r0, #0xff
    /* ISRs start at 16, so substract 16 to get zero-indexed */
    sub r0, #16


    /* r1 = NVIC.ICER[r0 / 32] */
    mov r1, #0xe180
    movt r1, #0xe000
    lsr r2, r0, #5
    mov r3, #4
    mul r2, r3
    add r1, r2

    /* r2 = 1 << (r0 & 31) */
    mov r2, #1
    mov r3, #31
    and r3, r0
    lsl r2, r2, r3

    /* *r1 = r2 */
    str r2, [r1, #0]");
}

#[cfg(not(target_os = "none"))]
#[allow(non_snake_case)]
pub unsafe extern "C" fn SVC_Handler() {}

#[cfg(target_os = "none")]
#[no_mangle]
#[naked]
#[allow(non_snake_case)]
pub unsafe extern "C" fn SVC_Handler() {
    asm!("
  cmp lr, #0xfffffff9
  bne to_kernel

  /* Set thread mode to unprivileged */
  mov r0, #1
  msr CONTROL, r0

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

  movw LR, #0xFFF9
  movt LR, #0xFFFF");
}

#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn switch_to_user(user_stack: *const u8, process_got: *const u8) -> *mut u8 {
    user_stack as *mut u8
}

#[cfg(target_os = "none")]
#[no_mangle]
#[inline(never)]
/// r0 is top of user stack, r1 Process GOT
pub unsafe extern "C" fn switch_to_user(mut user_stack: *const u8,
                                        process_got: *const u8,
                                        process_regs: &mut [usize; 8])
                                        -> *mut u8 {
    asm!("
    /* Load non-hardware-stacked registers from Process stack */
    ldmia $3, {r4-r11}
    /* Load bottom of stack into Process Stack Pointer */
    msr psp, $0

    /* Set PIC base pointer to the Process GOT */
    mov r9, $2

    /* Ensure that $3 is stored in a callee saved register */
    mov r0, $3

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

    /* Push non-hardware-stacked registers into Process struct's */
    /* regs field */
    stmia r0, {r4-r11}

    mrs $0, PSP /* PSP into r0 */"
    : "=r"(user_stack)
    : "r"(user_stack), "r"(process_got), "r"(process_regs)
    : "r4","r5","r6","r7","r8","r9","r10","r11");
    user_stack as *mut u8
}
