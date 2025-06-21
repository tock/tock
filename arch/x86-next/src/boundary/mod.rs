// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Usermode-kernelmode boundary for the x86 architecture
//!
//! This module defines the boundary between user and kernel modes on the x86 architecture,
//! including syscall/upcall calling conventions as well as process initialization.
//!
//! In contrast with other embedded architectures like ARM or RISC-V, x86 does _not_ have very many
//! general purpose registers to spare. The ABI defined here draws heavily from the `cdecl` calling
//! convention by using the stack instead of registers to pass data between user and kernel mode.
//!
//! ## System Calls
//!
//! System calls are dispatched from user- to kernel-mode using interrupt number 0x40. The system
//! call class number is stored in EAX.
//!
//! ECX and EDX are treated as caller-saved registers, which means their values should be considered
//! undefined after the return of a system call. The kernel itself will backup and restore ECX/EDX,
//! however they may get clobbered if an upcall is pushed.
//!
//! Arguments and return values are passed via the user-mode stack in reverse order (similar to
//! `cdecl`). The caller must _always_ push 4 values to the stack, even for system calls which have
//! less than 4 arguments. This is because the kernel may return up to 4 values.
//!
//! As with `cdecl`, the caller is responsible for incrementing ESP to clean up the stack frame
//! after the system call returns.
//!
//! The following assembly snippet shows how to invoke the `yield` syscall:
//!
//! ```text
//! push    0           # arg 4: unused
//! push    0           # arg 3: unused
//! push    0           # arg 2: unused
//! push    1           # arg 1: yield-wait
//! mov     eax, 0      # class: yield
//! int     0x40
//! add     esp, 16     # clean up stack
//! ```
//!
//! ## Yielding and Upcalls
//!
//! Upcalls are expected to be standard `cdecl` functions. The kernel will write arguments and a
//! return value to the stack before jumping to an upcall.
//!
//! The return address pushed by the kernel will point to the instruction immediately following the
//! `yield` syscall which led to the upcall. This means when the upcall finishes and returns, the
//! app will continue executing wherever it left off when `yield` was called, without context
//! switching back to kernel.
//!
//! The app should have allocated 16 bytes of stack space for arguments to/return values from the
//! `yield` syscall (see above). Since `yield` does not actually return anything, this stack space
//! is repurposed to store the upcall arguments. This way when an upcall returns, the app only needs
//! to clean up 16 bytes of stack space; in this sense, the ABI of `yield` is no different than any
//! other syscall.
//!
//! ## Process Initialization
//!
//! Tock treats process entry points just like upcalls, except they are never expected to return.
//! For x86, this means the process entry point should use the `cdecl` function.
//!
//! For x86, we allocate an initial stack of 36 bytes before calling this upcall. The top 20 bytes
//! are used to invoke the entry point itself: 16 for arguments, and 4 for a return address (which
//! should never be used).
//!
//! The remaining 16 bytes are free for use by the entry point. This is exactly enough stack space
//! to invoke system calls. In most cases, the first task of the app entry point will be to allocate
//! itself a larger stack.

mod context;
use context::UserContext;

mod boundary_impl;
pub use self::boundary_impl::Boundary;

#[cfg(target_arch = "x86")]
mod switch_to_user;

#[cfg(target_arch = "x86")]
mod return_from_user;

extern "cdecl" {
    /// Performs a context switch to the given process.
    ///
    /// See _switch_to_user.s_ for complete details.
    fn switch_to_user(context: *mut UserContext, error_code: *mut u32) -> u32;
}
