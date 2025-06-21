// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

// Context switching from kernel to user mode
//
// This routine implements context switching from kernel mode (ring 0) to user mode (ring 3).
//
// This is a standard cdecl routine that can be called directly from Rust code. It expects the
// following arguments:
//
// [esp+8]       *mut u32
//               If the user process generates a CPU exception with an associated error code, that
//               error code is written to the location provided here. Otherwise, a value of 0 is
//               written.
//
// [esp+4]       *mut UserContext
//               Pointer to a UserContext instance which holds the state of the user mode process
//               which we are switching to.
//
// At a high-level, context switching is carried out as follows:
//
// 1. The current CPU state of the kernel is stored on the stack. This includes general purpose as
//    well as segment registers.
// 2. After storing kernel state on the stack, the current stack pointer is stored in the "esp0"
//    field of the global TSS. This allows the hardware to switch back to the kernel stack when a
//    user app is interrupted.
// 3. We restore the user app's state using values from the given UserContext, then execute an iretd
//    instruction which jumps to the user app in ring 3.
//
// Once active, the user app will continue to execute until an interrupt occurs. This could be due to
// a hardware interrupt, system call, or CPU exception. In any of these cases, control passes to
// return_from_user, which performs the inverse of the above steps and returns control back to
// the original caller of switch_to_user. See return_from_user.rs for more information.

use core::arch::global_asm;

global_asm!(
    "
.section .text
.global switch_to_user
.global _switch_to_user
switch_to_user:
_switch_to_user:

    # Save kernel state
    push    edi
    push    esi
    push    ebp
    push    ebx

    # Manually push 16-bit segment registers. Using a real push instruction seems to have
    # inconsistent behavior (some of the registers move the stack by 2 bytes, others by 4).
    sub     esp, 4
    mov     [esp], ds
    sub     esp, 4
    mov     [esp], es
    sub     esp, 4
    mov     [esp], fs
    sub     esp, 4
    mov     [esp], gs

    # Store current ESP for ring 0 into the global TSS. This ensures the CPU switches back to the
    # kernel stack when an interrupt occurs.
    push    esp
    call    set_tss_esp0
    add     esp, 4

    mov     eax, dword ptr [esp+36]           # UserContext

    # Prepare stack for iretd to user mode
    push    dword ptr [eax+44]                 # SS
    push    dword ptr [eax+28]                 # ESP
    push    dword ptr [eax+36]                 # EFLAGS
    push    dword ptr [eax+40]                 # CS
    push    dword ptr [eax+32]                 # EIP

    # Restore CPU state (except for EAX as this holds our UserContext pointer)
    mov     ebx, dword ptr [eax+4]
    mov     ecx, dword ptr [eax+8]
    mov     edx, dword ptr [eax+12]
    mov     esi, dword ptr [eax+16]
    mov     edi, dword ptr [eax+20]
    mov     ebp, dword ptr [eax+24]
    mov     es, [eax+52]
    mov     fs, [eax+56]
    mov     gs, [eax+60]

    # Stash EAX on the stack, because we won't be able to access it after switching DS
    push    dword ptr [eax]

    # Switch to user data segment
    mov     ds, [eax+48]

    # Now restore EAX from the stack
    pop     eax

    iretd"
);
