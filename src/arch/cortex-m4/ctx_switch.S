.cpu cortex-m4
.syntax unified
.thumb
.text

/* Exported functions */
.global SVC_Handler
.globl switch_to_user
.globl systick_handler

.extern OVERFLOW_FIRED

.thumb_func
SVC_Handler:
  cmp lr, #0xfffffff9
  bne to_kernel
  movw lr, #0xfffd
  movt lr, #0xffff
  bx lr
to_kernel:
  movw LR, #0xFFF9
  movt LR, #0xFFFF
  bx lr

.thumb_func
/* r0 is top of user stack, r1 Process GOT */
switch_to_user:
    push {r4-r11}

    /* Load non-hardware-stacked registers from Process stack */
    sub r0, #32
    ldmia r0!, {r4-r11}
    /* Load bottom of stack into Process Stack Pointer */
    msr psp, r0

    /* Set PIC base pointer to the Process GOT */
    mov r9, r1

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

    mrs r0, PSP /* PSP into r0 */

    /* Push non-harware-stacked registers onto Process stack */
    /* r0 points to user stack (see to_kernel) */
    stmdb r0, {r4-r11}

    pop {r4-r11}
    bx lr

.thumb_func
systick_handler:
  ldr r0, =OVERFLOW_FIRED
  mov r1, #1
  str r1, [r0, #0]
  movw LR, #0xFFF9
  movt LR, #0xFFFF
  bx lr
