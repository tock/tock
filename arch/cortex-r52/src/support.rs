// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Low-level support functions for Cortex-R52.

use core::arch::asm;
use core::fmt::Write;
use kernel::platform::chip::ThreadIdProvider;

/// No-operation instruction used for busy-wait loops.
#[inline(always)]
pub fn nop() {
    unsafe { asm!("nop", options(nomem, nostack, preserves_flags)) }
}

/// Wait-for-interrupt instruction to lower power consumption in idle loops.
#[inline(always)]
pub fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack, preserves_flags)) }
}

/// Generic word irq is used but this is for FIQ!
pub unsafe fn irq_enable() {
    // Enable FIQ in PSTATE
    asm!("cpsie f", options(nomem, nostack));
}

/// Generic word irq is used but this is for FIQ!
pub unsafe fn irq_disable() {
    // Disable FIQ in PSTATE
    asm!("cpsid f", options(nomem, nostack));
}

pub fn with_interrupts_disabled<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe {
        irq_disable();
    }
    let res = f();
    unsafe {
        irq_enable();
    }
    res
}

/// Implement the [`ThreadIdProvider`] trait for Cortex-R52 platforms.
pub enum CortexR52ThreadIdProvider {}

unsafe impl ThreadIdProvider for CortexR52ThreadIdProvider {
    fn running_thread_id() -> usize {
        crate::support::is_interrupt_context() as usize
    }
}

/// TODO: This implementation is probably broken. I have no idea what
/// it should do and I just adapted it from Cortex M; the PE runs in:
///  - USR mode for apps
///  - SVC mode for kernel
///  - FIQ mode after an IRQ, but switch back very quickly to SVC
/// ...so it will always returns 0!
pub fn is_interrupt_context() -> bool {
    use core::arch::asm;
    let mut mode: u32;

    unsafe {
        asm!(
            "mrs r0, cpsr",
            out("r0") mode,
            options(nomem, nostack, preserves_flags)
        );
    }

    // If CPSR[7:0] is 0 then we are in USR mode.
    (mode & 0xF) != 0
}

pub unsafe fn print_state(_writer: &mut dyn Write) {
    todo!()
    //let p_mext = csr::CSR.mip.is_set(csr::mip::mip::mext);
    //let _ = writer.write_fmt(format_args!(
    //    "\r\n mie:   {:#010X}   mip:   {:#010X}\
    //    ...
    //     \r\n  mext:   {:6}              {:6}\r\n",
    //    csr::CSR.mie.get(),
    //    ...
    //    p_mext
    //));
}
