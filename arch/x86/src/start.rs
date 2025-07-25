// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::arch::naked_asm;

#[unsafe(link_section = ".x86.start")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() {
    naked_asm!(
        "
    # Initialize the stack
    lea     esp, _estack
    lea     ebp, _estack

    # Zero out the .bss section
    mov     eax, _szero
    mov     ebx, _ezero
200:
    cmp     eax, ebx
    je      201f
    mov     byte ptr [eax], 0
    inc     eax
    jmp     200b
201:

    # Initialize contents of the .data section
    mov     eax, _srelocate
    mov     ebx, _erelocate
    mov     ecx, _etext
300:
    cmp     eax, ebx
    je      301f
    mov     dl, byte ptr [ecx]
    mov     byte ptr [eax], dl
    inc     eax
    inc     ecx
    jmp     300b
301:

    # Now we hand control over to the Rust main function
    call    main

    # main should never return, but just in case:
3:
    hlt
    jmp     3b

"
    );
}
