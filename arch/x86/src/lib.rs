// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Generic support for 32-bit Intel/AMD CPUs.
//!
//! ## Interrupt Handling
//!
//! Once initialized, this crate assumes ownership of the CPU's interrupt handling facilities.
//!
//! Interrupts from external devices are handled by calling an extern function named
//! `_handle_external_interrupt`, which must be defined by the chip or board crate. This function
//! must have a `cdecl` ABI and must accept a single `u32` argument which is the number of the
//! external interrupt.
//!
//! The `_handle_external_interrupt` function should typically perform the following tasks:
//!
//! 1. Call [`InterruptPoller::set_pending`] to mark the given interrupt as pending
//! 2. Send an EOI signal to any relevant interrupt controllers
//!
//! Any other logic needed to service the interrupt (for instance, reading from buffers or re-arming
//! hardware) should typically be performed within the current chip's `service_pending_interrupts`
//! method.
//!
//! Apart from external interrupts, all other interrupt categories such as CPU exceptions or system
//! calls are handled internally by this crate.
//!
//! ## Safety
//!
//! Some of the `unsafe` code in this crate relies on the blanket assumption that this code is being
//! compiled into a Tock kernel for an x86 system. When calling code from this crate, the following
//! statements must always be true:
//!
//! * The CPU is executing at ring 0
//! * The CPU has I/O privileges

#![deny(unsafe_op_in_unsafe_fn)]
#![no_std]
mod boundary;
pub use boundary::Boundary;

mod interrupts;
pub use interrupts::InterruptPoller;
pub use interrupts::IDT_RESERVED_EXCEPTIONS;

mod segmentation;

pub mod support;

pub mod mpu;

pub mod registers;

#[cfg(target_arch = "x86")]
mod start;

/// Performs low-level CPU initialization.
///
/// This function installs new segmentation and interrupt handling regimes which the rest of this
/// crate needs to function properly.
///
/// ## Safety
///
/// This function must never be executed more than once.
///
/// Before calling, memory must be identity mapped. Otherwise the introduction of flat segmentation
/// will cause the kernel's code/data to move unexpectedly.
///
/// After this function returns, it is safe to enable interrupts. However, interrupts below number
/// 32 must **never** be generated except by the CPU itself (i.e. exceptions), as doing so would
/// interfere with the internal handler stubs. This means that before enabling interrupts, the
/// caller must ensure that any hardware delivering external interrupts (such as the PIC/APIC) is
/// configured to use interrupt number 32 or above.
pub unsafe fn init() {
    unsafe {
        segmentation::init();
        interrupts::init();
    }
}

/// Stops instruction execution and places the processor in a HALT state.
///
/// An enabled interrupt (including NMI and SMI), a debug exception, the BINIT#
/// signal, the INIT# signal, or the RESET# signal will resume execution. If an
/// interrupt (including NMI) is used to resume execution after a HLT instruction,
/// the saved instruction pointer (CS:EIP) points to the instruction following
/// the HLT instruction.
///
/// # Safety
/// Will cause a general protection fault if used outside of ring 0.
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn halt() {
    use core::arch::asm;

    unsafe {
        asm!("hlt", options(att_syntax, nomem, nostack)); // check if preserves_flags
    }
}
