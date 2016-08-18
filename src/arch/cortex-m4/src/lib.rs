#![crate_name = "cortexm4"]
#![crate_type = "rlib"]
#![feature(asm,const_fn,naked_functions)]
#![no_std]

extern crate common;
extern crate main;

pub mod mpu;
pub mod systick;

#[no_mangle]
#[naked]
pub unsafe extern fn systick_handler() {
    asm!("
        /* Skip saving process state if not coming from user-space */
        cmp lr, #0xfffffffd
        bne _systick_handler_no_stacking

        mrs r0, PSP /* PSP into r0 */

        /* Push non-hardware-stacked registers onto Process stack */
        /* r0 points to user stack (see to_kernel) */
        stmdb r0, {r4-r11}
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


#[no_mangle]
#[naked]
/// All ISRs are caught by this handler which indirects to a custom handler by
/// indexing into `INTERRUPT_TABLE` based on the ISR number.
pub unsafe extern fn generic_isr() {
    asm!("
    /* Skip saving process state if not coming from user-space */
    cmp lr, #0xfffffffd
    bne _ggeneric_isr_no_stacking

    mrs r0, PSP /* PSP into r0 */

    /* Push non-hardware-stacked registers onto Process stack */
    /* r0 points to user stack (see to_kernel) */
    stmdb r0, {r4-r11}
_ggeneric_isr_no_stacking:
    /* Find the ISR number by looking at the low byte of the IPSR registers */
    mrs r0, IPSR
    and r0, #0xff
    /* ISRs start at 16, so substract 16 to get zero-indexed */
    sub r0, #16

    /* INTERRUPT_TABLE contains function pointers, which are word sized, so
     * multiply by 4 (the word size) */
    lsl r0, r0, #2

    ldr r1, =INTERRUPT_TABLE
    ldr r0, [r1, r0]

    push {lr}
    blx r0
    pop {lr}

    /* Set thread mode to privileged */
    mov r0, #0
    msr CONTROL, r0

    movw LR, #0xFFF9
    movt LR, #0xFFFF");
}

#[no_mangle]
#[naked]
#[allow(non_snake_case)]
pub unsafe extern fn SVC_Handler() {
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

#[no_mangle]
#[inline(never)]
/// r0 is top of user stack, r1 Process GOT
pub unsafe extern fn switch_to_user(mut user_stack: *const u8, process_got: *const u8) -> *mut u8 {
    asm!("
    /* Load non-hardware-stacked registers from Process stack */
    sub $0, #32
    ldmia $0!, {r4-r11}
    /* Load bottom of stack into Process Stack Pointer */
    msr psp, $0

    /* Set PIC base pointer to the Process GOT */
    mov r9, $2

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

    mrs $0, PSP /* PSP into r0 */

    /* Push non-hardware-stacked registers onto Process stack */
    /* r0 points to user stack (see to_kernel) */
    stmdb $0, {r4-r11}"
    : "=r"(user_stack)
    : "r"(user_stack), "r"(process_got)
    : "r4","r5","r6","r7","r8","r9","r10","r11");
    user_stack as *mut u8
}

