.cpu cortex-m4
.syntax unified
.thumb
.text

/* Exported functions */
.global SVC_Handler
.globl switch_to_user
.globl generic_isr
.globl systick_handler

.extern INTERRUPT_TABLE
.extern SYSCALL_FIRED
.extern OVERFLOW_FIRED

.thumb_func
SVC_Handler:
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

    ldr r0, =SYSCALL_FIRED
    mov r1, #0
    str r1, [r0, #0]

    /* SWITCH */
    svc 0xff /* It doesn't matter which SVC number we use here */

    mrs r0, PSP /* PSP into r0 */

    /* Push non-hardware-stacked registers onto Process stack */
    /* r0 points to user stack (see to_kernel) */
    stmdb r0, {r4-r11}

    pop {r4-r11}
    bx lr

.thumb_func
/* All ISRs are caught by this handler which indirects to a custom handler by
 * indexing into `INTERRUPT_TABLE` based on the ISR number. */
generic_isr:
    /* Skip saving process state if not coming from user-space */
    cmp lr, #0xfffffffd
    bne _generic_isr_no_stacking

    mrs r0, PSP /* PSP into r0 */

    /* Push non-hardware-stacked registers onto Process stack */
    /* r0 points to user stack (see to_kernel) */
    stmdb r0, {r4-r11}
_generic_isr_no_stacking:
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
    movt LR, #0xFFFF
    bx lr

.thumb_func
systick_handler:
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
    bx lr

