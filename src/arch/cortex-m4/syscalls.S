.cpu cortex-m4
.syntax unified
.thumb
.text

.section .syscalls

.global __wait
.global __subscribe
.global __command

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

