.cpu cortex-m4
.syntax unified
.thumb
.text

/* Exported functions */
.global SVC_Handler
.globl switch_to_user

/* SVC wrappers */
.globl __subscribe
.globl __command
.globl __wait

/* svc_rust_handler returns 0 to return to unprivileged stack (process) and 1
  to switch to privileged stack (kernel) */
.extern svc_rust_handler
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
/* r0 is start address and r1 is top of user stack */
switch_to_user:
    /* Load bottom of stack into Process Stack Pointer */
    msr psp, r0

    push {r4-r11}
    svc 0xff
    pop {r4-r11}

    bx lr

.thumb_func
__wait:
    push {r4-r11,lr}
    svc 0
    pop {r4-r11,lr}
    bx lr

.thumb_func
__subscribe:
    push {r4-r11}
    svc 1
    pop {r4-r11}
    bx lr

.thumb_func
__command:
    push {r4-r11}
    svc 2
    pop {r4-r11}
    bx lr

