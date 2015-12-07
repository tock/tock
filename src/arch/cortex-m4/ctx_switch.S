.cpu cortex-m4
.syntax unified
.thumb
.text

/* Exported functions */
.global SVC_Handler
.globl switch_to_user

.thumb_func
SVC_Handler:
  cmp lr, #0xfffffff9
  bne to_kernel
  movw lr, #0xfffd
  movt lr, #0xffff
  bx lr
to_kernel:
  mrs r0, PSP /* PSP into r0 */
  str r0, [sp, #0] /* PSP into Master stack r0 */
  movw LR, #0xFFF9
  movt LR, #0xFFFF
  bx lr

.thumb_func
/* r0 is top of user stack, r1 is heap base */
switch_to_user:
    /* Load bottom of stack into Process Stack Pointer */
    msr psp, r0

    push {r4-r11}
    mov r9, r1
    svc 0xff
    pop {r4-r11}
    bx lr

