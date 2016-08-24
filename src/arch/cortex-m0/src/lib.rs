#![crate_name = "cortexm0"]
#![crate_type = "rlib"]
#![feature(asm,const_fn,naked_functions)]
#![no_std]

extern crate common;
extern crate main;

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
/// r0 is top of user stack, r1 Process GOT
pub unsafe extern "C" fn switch_to_user(mut user_stack: *const u8,
                                        process_got: *const u8)
                                        -> *mut u8 {
    asm!("
    /* Load non-hardware-stacked registers from Process stack */
    subs $0, #32
    ldmia $0!, {r4-r7}
    mov r11, r7
    mov r10, r6
    mov r9,  r5
    mov r8,  r4
    ldmia $0!, {r4-r7}

    /* Load bottom of stack into Process Stack Pointer */
    msr psp, $0

    /* Set PIC base pointer to the Process GOT */
    mov r9, $2

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

    mrs $0, PSP /* PSP into r0 */

    /* Push non-hardware-stacked registers onto Process stack */
    /* r0 points to user stack (see to_kernel) */
    subs r0, #32 // We actually store r4-r11 below the process stack
    str r4, [$0, #16]
    str r5, [$0, #20]
    str r6, [$0, #24]
    str r7, [$0, #28]

    mov  r4, r8
    mov  r5, r9
    mov  r6, r10
    mov  r7, r11

    str r4, [$0, #0]
    str r5, [$0, #4]
    str r6, [$0, #8]
    str r7, [$0, #12]
    adds $0, #32 // We actually store r4-r11 below the process stack
    "
    : "=r"(user_stack)
    : "r"(user_stack), "r"(process_got)
    : "r4","r5","r6","r7","r8","r9","r10","r11");
    user_stack as *mut u8
}
