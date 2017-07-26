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
#[inline(never)]
pub unsafe extern "C" fn switch_to_user(mut user_stack: *const u8,
                                        process_regs: &mut [usize; 8])
                                        -> *mut u8 {
    asm!("
    /* Load non-hardware-stacked registers from Process stack */
    ldmia $2!, {r4-r7}
    mov r11, r7
    mov r10, r6
    mov r9,  r5
    mov r8,  r4
    ldmia $2!, {r4-r7}
    subs $2, 32 /* Restore pointer to process_regs
                /* ldmia! added a 32-byte offset */

    /* Load bottom of stack into Process Stack Pointer */
    msr psp, $0

    /* Ensure process_regs is stored in a hardware stacked register */
    /* so we have it when we return from SVC */
    mov r0, $2

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

    /* Store non-hardware-stacked registers in process_regs */
    /* r0 points to user stack */
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
    : "r"(user_stack), "r"(process_regs)
    : "r4","r5","r6","r7","r8","r9","r10","r11");
    user_stack as *mut u8
}
