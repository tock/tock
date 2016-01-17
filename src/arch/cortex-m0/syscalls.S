.cpu cortex-m0
.syntax unified
.thumb
.text

.section .syscalls

/* Cortex-M0 can only push/pop registers R0-R7 directly, so move R8-R11 to/from
 * R0-R3. This is equivalent to the 32-bit "push/pop {r4-r11}" instructions. */

.macro SYSCALL NAME, NUM
.global \NAME
.thumb_func
\NAME :
  push {r4-r7}
  mov r4,  r8
  mov r5,  r9
  mov r6, r10
  mov r7, r11
  push {r4-r7}
  svc \NUM
  pop {r4-r7}
  mov  r8, r4
  mov  r9, r5
  mov r10, r6
  mov r11, r7
  pop {r4-r7}
  bx lr
.endm

SYSCALL __wait, 0
SYSCALL __subscribe, 1
SYSCALL __command, 2
SYSCALL __allow, 3
