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
.ifc \NAME,__wait
  push {lr}
.endif
  svc \NUM
.ifc \NAME,__wait
  pop {pc}
.else
  bx lr
.endif
.endm

SYSCALL __wait, 0
SYSCALL __subscribe, 1
SYSCALL __command, 2
SYSCALL __allow, 3
SYSCALL __memop, 4
