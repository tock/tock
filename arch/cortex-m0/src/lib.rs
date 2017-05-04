#![feature(asm,const_fn,naked_functions)]
#![no_std]

extern crate kernel;

#[no_mangle]
#[naked]
#[allow(non_snake_case)]
pub unsafe extern "C" fn SVC_Handler() {
    asm!("
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

EXC_RETURN_MSP:
  .word 0xFFFFFFF9
EXC_RETURN_PSP:
  .word 0xFFFFFFFD
  ");
}

#[no_mangle]
/// All ISRs are caught by this handler which indirects to a custom handler by
/// indexing into `INTERRUPT_TABLE` based on the ISR number.
pub unsafe extern "C" fn generic_isr() {
    asm!("
    /* Skip saving process state if not coming from user-space */
    ldr r0, EXC_RETURN_PSP
    cmp lr, r0
    bne _ggeneric_isr_no_stacking

    /* We need the most recent kernel's version of r0, which points */
    /* to the Process struct's stored registers field. The kernel's r0 */
    /* lives in the first word of the hardware stacked registers on MSP */
    mov r0, sp
    ldr r0, [r0, #0]

    /* Push non-hardware-stacked registers onto Process stack */
    /* r0 points to user stack (see to_kernel) */
    str r4, [r0, #16]
    str r5, [r0, #20]
    str r6, [r0, #24]
    str r7, [r0, #28]

    mov  r4, r8
    mov  r5, r9
    mov  r6, r10
    mov  r7, r11

    str r4, [r0, #0]
    str r5, [r0, #4]
    str r6, [r0, #8]
    str r7, [r0, #12]

_ggeneric_isr_no_stacking:
    /* Find the ISR number by looking at the low byte of the IPSR registers */
    mrs r0, IPSR
    movs r1, #0xff
    ands r0, r1
    /* ISRs start at 16, so substract 16 to get zero-indexed */
    subs r0, #16

    /* INTERRUPT_TABLE contains function pointers, which are word sized, so
     * multiply by 4 (the word size) */
    lsls r0, r0, #2

    ldr r1, =INTERRUPT_TABLE
    ldr r0, [r1, r0]

    push {lr}
    blx r0
    /* pop {lr} */
    pop {r0}
    mov lr, r0

    /* Set thread mode to privileged */
    movs r0, #0
    msr CONTROL, r0

    ldr r0, EXC_RETURN_PSP
    mov lr, r0
    ");
}

#[no_mangle]
#[inline(never)]
/// r0 is top of user stack, r1 Process GOT
pub unsafe extern "C" fn switch_to_user(mut user_stack: *const u8,
                                        process_got: *const u8,
                                        process_regs: &mut [usize; 8])
                                        -> *mut u8 {
    asm!("
    /* Load non-hardware-stacked registers from Process stack */
    ldmia $3!, {r4-r7}
    mov r11, r7
    mov r10, r6
    mov r9,  r5
    mov r8,  r4
    ldmia $3!, {r4-r7}
    subs $3, 32

    /* Load bottom of stack into Process Stack Pointer */
    msr psp, $0

    /* Set PIC base pointer to the Process GOT */
    mov r9, $2

    mov r0, $3

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

    /* Push non-hardware-stacked registers onto Process stack */
    /* r0 points to user stack (see to_kernel) */
    str r4, [r0, #16]
    str r5, [r0, #20]
    str r6, [r0, #24]
    str r7, [r0, #28]

    mov  r4, r8
    mov  r5, r9
    mov  r6, r10
    mov  r7, r11

    str r4, [r0, #0]
    str r5, [r0, #4]
    str r6, [r0, #8]
    str r7, [r0, #12]

    mrs $0, PSP /* PSP into r0 */

    "
    : "=r"(user_stack)
    : "r"(user_stack), "r"(process_got), "r"(process_regs)
    : "r4","r5","r6","r7","r8","r9","r10","r11");
    user_stack as *mut u8
}
