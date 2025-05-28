// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

// Context switching from user back to kernel mode
//
// This routine is always jumped to (not called) by the common interrupt handler, handler_entry. See
// handler_entry.rs for an explanation of the stack layout when this routine is called.
//
// Additionally, even higher up the stack we can expect to find the following values which were
// placed by switch_to_user (see switch_to_user.s):
//
// [esp+80]      *mut u32 (error code, second arg of switch_to_user)
// [esp+76]      *mut UserContext (first arg of switch_to_user)
// [esp+72]      Return address (from when switch_to_user was called)
// [esp+68]      Kernel EDI
// [esp+64]      Kernel ESI
// [esp+60]      Kernel EBP
// [esp+56]      Kernel EBX
// [esp+52]      Kernel DS
// [esp+48]      Kernel ES
// [esp+44]      Kernel FS
// [esp+40]      Kernel GS
//
// Roughly, this routine performs the inverse of switch_to_user. It stores the current CPU state into
// the UserContext of the current process, restores kernel CPU state from the stack, and then returns
// control to the original caller of switch_to_user.
//
// 1. The current CPU state is stored into the UserContext of the current process.
// 2. Kernel state is restored from the stack.
// 3. Control is returned to the original caller of switch_to_user.
//
// From the perspective of the code that originally called switch_to_user, it should look like a
// regular cdecl function call occurred.

use core::arch::global_asm;

global_asm!(
    "
    .section .text
    .global return_from_user
    return_from_user:

        mov     ecx, dword ptr [esp+76]       # UserContext

        # First switch back to the kernel's data segment. Once this is done, we are safe to start
        # storing things in UserContext.
        mov     eax, ds
        mov     edx, [esp+52]
        mov     ds, edx
        mov     dword ptr [ecx+48], eax

        # Store the remaining general purpose registers
        mov     dword ptr [ecx+4], ebx
        mov     dword ptr [ecx+16], esi
        mov     dword ptr [ecx+20], edi
        mov     dword ptr [ecx+24], ebp

        # Store segment selectors
        mov     eax, es
        mov     dword ptr [ecx+52], eax
        mov     eax, fs
        mov     dword ptr [ecx+56], eax
        mov     eax, gs
        mov     dword ptr [ecx+60], eax

        mov     edx, dword ptr [esp+80]       # Load error code pointer

        # Then unwind the stack
        pop     dword ptr [ecx+12]             # EDX
        pop     dword ptr [ecx+8]              # ECX
        pop     dword ptr [ecx]                # EAX
        pop     eax                            # Interrupt number (returned in EAX)
        pop     dword ptr [edx]                # Error code
        pop     dword ptr [ecx+32]             # EIP
        pop     dword ptr [ecx+40]             # CS
        pop     dword ptr [ecx+36]             # EFLAGS
        pop     dword ptr [ecx+28]             # ESP
        pop     dword ptr [ecx+44]             # SS

        # Manually pop 16-bit segment registers. Using a real pop instruction seems to have
        # inconsistent behavior (some of the registers move the stack by 2 bytes, others by 4).
        mov     gs, [esp]
        add     esp, 4
        mov     fs, [esp]
        add     esp, 4
        mov     es, [esp]
        add     esp, 4
        # Already restored DS above, so we only need to increment ESP
        add     esp, 4

        # Restore remaining kernel-mode CPU state
        pop     ebx
        pop     ebp
        pop     esi
        pop     edi

        # Return to whoever called switch_to_user
        ret
"
);
