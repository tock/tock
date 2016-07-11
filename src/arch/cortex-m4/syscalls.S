.cpu cortex-m4
.syntax unified
.thumb
.text

.section .syscalls

.global __wait
.global __allow
.global __subscribe
.global __command
.global __memop

.thumb_func
__wait:
    push {lr}
    svc 0
    pop {pc}

.thumb_func
__allow:
    svc 3
    bx lr

.thumb_func
__subscribe:
    svc 1
    bx lr

.thumb_func
__command:
    svc 2
    bx lr

.thumb_func
__memop:
    svc 4
    bx lr

