.cpu cortex-m0
.syntax unified
.thumb
.text

/* Exported functions */
.global SVC_Handler
.globl switch_to_user

.thumb_func
SVC_Handler:
  ldr r0, EXC_RETURN_MSP
  cmp lr, r0
  bne to_kernel
  ldr r1, EXC_RETURN_PSP
  bx r1

to_kernel:
  mrs r0, PSP /* PSP into r0 */
  str r0, [sp, #0] /* PSP into Master stack r0 */
  ldr r1, EXC_RETURN_MSP
  bx r1

EXC_RETURN_MSP:
  .word 0xFFFFFFF9
EXC_RETURN_PSP:
  .word 0xFFFFFFFD

.thumb_func
/* r0 is top of user stack, r1 Process GOT */
switch_to_user:
    /* Cortex-M0 can only push registers R0-R7 directly, so move R8-R11 to R0-R3.
     * This is equivalent to the 32-bit "push {r4-r11}" instruction. */
    push {r4-r7}
    mov r4,  r8
    mov r5,  r9
    mov r6, r10
    mov r7, r11
    push {r4-r7}

    /* Load non-hardware-stacked registers from Process stack */
    subs r0, #32
    ldr r7, [r0, #0]
    ldr r6, [r0, #4]
    ldr r5, [r0, #8]
    ldr r4, [r0, #12]
    mov r4,  r8
    mov r5,  r9
    mov r6, r10
    mov r7, r11
    ldr r7, [r0, #16]
    ldr r6, [r0, #20]
    ldr r5, [r0, #24]
    ldr r4, [r0, #28]
    adds r0, #32
    /* Load bottom of stack into Process Stack Pointer */
    msr psp, r0

    /* Set PIC base pointer to the Process GOT */
    mov r9, r1

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

    /* Push non-harware-stacked registers onto Process stack.  r0 points to user
     * stack (see to_kernel). We store the registers passed the stack to make it
     * easier to address hardware-stacked registers in the rest of the kernel.
     * This is fine since we just account for that above.
     */
    str r4, [r0, #32]
    str r5, [r0, #28]
    str r6, [r0, #24]
    str r7, [r0, #20]

    mov  r8, r4
    mov  r9, r5
    mov r10, r6
    mov r11, r7

    str r4, [r0, #16]
    str r6, [r0, #8]
    str r5, [r0, #12]
    str r7, [r0, #4]

    /* These instructions are equivalent to the 32-bit "pop {r4-r11}" */
    pop {r4-r7}
    mov  r8, r4
    mov  r9, r5
    mov r10, r6
    mov r11, r7
    pop {r4-r7}

    bx lr

