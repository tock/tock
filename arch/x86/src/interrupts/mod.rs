// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Facilities for handling interrupts and CPU exceptions

mod handlers;
mod idt;
mod poller;

pub use self::poller::InterruptPoller;

#[cfg(target_arch = "x86")]
mod handler_stubs;

#[cfg(target_arch = "x86")]
mod handler_entry;

/// Total number of interrupt vectors.
pub const NUM_VECTORS: usize = 256;

/// Interrupt number used for Tock system calls on x86.
pub const SYSCALL_VECTOR: u8 = 0x40;

/// Number of exceptions reserved in the IDT by Intel.
/// Reference: <https://en.wikipedia.org/wiki/Interrupt_descriptor_table#Common_IDT_layouts>
pub const IDT_RESERVED_EXCEPTIONS: u8 = 32;

/// Performs global initialization of interrupt handling.
///
/// After calling this function, [`InterruptPoller`] can be used to poll for and handle interrupts.
///
/// ## Safety
///
/// This function must never be executed more than once.
///
/// The kernel's segmentation must already be initialized (via [`segmentation::init`][crate::segmentation::init])
/// prior to calling this function, and it must never be changed afterwards.
///
/// After this function returns, it is safe to enable interrupts. However, interrupts below number
/// 32 ([`IDT_RESERVED_EXCEPTIONS`]) must **never** be generated except by the CPU itself (i.e. exceptions),
/// as doing so would interfere with the internal handler stubs. This means that before enabling interrupts,
/// the caller must ensure that any hardware delivering external interrupts (such as the PIC/APIC) is
/// configured to use interrupt number 32 or above.
pub(crate) unsafe fn init() {
    unsafe {
        idt::init();
    }
}
