// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Low-level support functions for Cortex-R52.

use core::arch::asm;
use core::ptr::self;
use core::fmt::Write;
use kernel::platform::chip::ThreadIdProvider;

/// No-operation instruction used for busy-wait loops.
#[inline(always)]
pub fn nop() {
    unsafe { asm!("nop", options(nomem, nostack, preserves_flags)) }
}

/// Wait-for-interrupt instruction to lower power consumption in idle loops.
/// The PE will wake up REGARDLESS of A,I,F flags in PSTATE.
#[inline(always)]
pub fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack, preserves_flags)) }
}

pub fn isb() {
    unsafe { asm!("isb", options(nomem, nostack, preserves_flags)) }
}

/// Data Synchronization Barrier with full system scope:
/// Outer Shareable for Reads and writes as per ARM DDI 0487
pub fn dsb_sy() {
    unsafe { asm!("dsb sy", options(nomem, nostack, preserves_flags)) }
}

// Generic word irq is used but this is for FIQ!
pub unsafe fn irq_enable() {
    // Enable FIQ in PSTATE
    asm!("cpsie f", options(nomem, nostack));
}

// Generic word irq is used but this is for FIQ!
pub unsafe fn is_irq_enabled() -> bool {
    let mut cpsr: u32;
    asm!(
        "mrs {0}, cpsr",
        out(reg) cpsr,
        options(nomem, nostack, preserves_flags)
    );
    (cpsr & (1 << 6)) == 0
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
///  - FIQ mode after an irq, but switch back very quickly to SVC
///
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


extern "C" {
    static _estack: usize;
}

#[unsafe(no_mangle)]
pub unsafe fn print_fault_frame() {

    sh::p!("\n--- EXCEPTION ---");

    let stack_top = core::ptr::addr_of!(_estack).cast::<u32>();
    //sh::p!("estack: {:8X}  sp: {:8X}", stack_top as u32, sp);

    for i in 0..7 {
        let a : u32 = ptr::read(stack_top.sub(14 - i - 0));
        let b : u32 = ptr::read(stack_top.sub(14 - i - 7));
        sh::p!("      R{:2}: {:8X}      R{:2}: {:8X}", i, a, i + 7, b);
    }

    let cpsr: u32 = ptr::read(stack_top.sub(15));
    let spsr: u32 = ptr::read(stack_top.sub(16));
    sh::p!("     cpsr: {:8X}     spsr: {:8X}\n", cpsr, spsr);

    let lr_svc: u32 = ptr::read(stack_top.sub(17));
    let sp_svc: u32 = ptr::read(stack_top.sub(18));
    let spsr_svc: u32 = ptr::read(stack_top.sub(19));
    sh::p!("   lr_svc: {:8X}   sp_svc: {:8X}\n spsr_svc: {:8X}\n", lr_svc, sp_svc, spsr_svc);

    let lr_usr: u32 = ptr::read(stack_top.sub(20));
    let sp_usr: u32 = ptr::read(stack_top.sub(21));
    sh::p!("   lr_usr: {:8X}   sp_usr: {:8X}\n", lr_usr, sp_usr);

    for i in 8..11 {
        let a : u32 = ptr::read(stack_top.sub(27 - i - 0));
        let b : u32 = ptr::read(stack_top.sub(27 - i - 3));
        sh::p!("  R{:2}_fiq: {:8X}  R{:2}_fiq: {:8X}", i, a, i + 3, b);
    }

    let sp_fiq: u32 = ptr::read(stack_top.sub(28));
    let spsr_fiq: u32 = ptr::read(stack_top.sub(29));
    sh::p!("   sp_fiq: {:8X} spsr_fiq: {:8X}\n", sp_fiq, spsr_fiq);

    let ifsr: u32 = ptr::read(stack_top.sub(30));
    let ifar: u32 = ptr::read(stack_top.sub(31));
    sh::p!("     ifsr: {:8X}     ifar: {:8X}", ifsr, ifar);

    let dfsr: u32 = ptr::read(stack_top.sub(32));
    let dfar: u32 = ptr::read(stack_top.sub(33));
    sh::p!("     dfsr: {:8X}     dfar: {:8X}", dfsr, dfar);

    let aifsr: u32 = ptr::read(stack_top.sub(34));
    let adfsr: u32 = ptr::read(stack_top.sub(35));
    sh::p!("    aifsr: {:8X}  adfsr: {:8X}", aifsr, adfsr);

    sh::p!("--- END ---\n");

    asm!("bkpt");
    loop {
        asm!("cpsid aif");
        asm!("wfe");
    }
}
