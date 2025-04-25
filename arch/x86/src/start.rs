// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::arch::global_asm;

global_asm!(
    "
.section .text

.global main

.global _start
_start:
    # Initialize the stack
    lea     esp, _estack
    lea     ebp, _estack

    # Zero out the .bss section
    mov     eax, _szero
    mov     ebx, _ezero
100:
    cmp     eax, ebx
    je      101f
    mov     byte ptr [eax], 0
    inc     eax
    jmp     100b
101:

    # Initialize contents of the .data section
    mov     eax, _srelocate
    mov     ebx, _erelocate
    mov     ecx, _etext
200:
    cmp     eax, ebx
    je      201f
    mov     dl, byte ptr [ecx]
    mov     byte ptr [eax], dl
    inc     eax
    inc     ecx
    jmp     200b
201:

    # Now we hand control over to the Rust main function
    call    main

    # main should never return, but just in case:
3:
    hlt
    jmp     3b

"
);
