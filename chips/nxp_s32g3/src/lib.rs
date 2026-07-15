// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

#![no_std]

pub mod chip;
pub mod clocks;
pub mod linflexd;
pub mod mc_me;
pub mod mscm;
pub mod siul2;
pub mod ssramc;
pub mod stm;
pub mod swt;
pub mod xrdc;

use cortexm7::unhandled_interrupt;
use cortexm7::{initialize_ram_jump_to_main, CortexM7, CortexMVariant};

// S32G3 reset handler for vector-fetch boot.
//
// Contract (unified vector-fetch semantics):
//   BASE_VECTORS[0] = _estack        → hardware loads as initial SP
//   BASE_VECTORS[1] = nxp_s32g3_boot_entry → hardware loads as reset PC
//
// The Cortex-M7 hardware performs a vector-fetch from `ram_start`
//   (SP = word0, PC = word1) before executing any instruction.
//
// With .multiboot empty, the Tock linker places BASE_VECTORS at byte 0 of
// the image (= ram_start), satisfying the boot ROM contract.
// The reset handler masks IRQs while it enables the FPU, zeros the L2 SRAM
// stack window, sets MSP, and restores PRIMASK before Tock RAM initialization.
#[cfg(any(doc, all(target_arch = "arm", target_os = "none")))]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nxp_s32g3_boot_entry() {
    use core::arch::naked_asm;

    naked_asm!(
        r#"
    /* 0. Mask IRQs. `cpsid i` protects the uninitialized ECC L2 SRAM stack
     *    window: an exception frame pushed before zeroing `_sstack .. _estack`
     *    could raise an imprecise BusFault. Reset entry has no caller; r4–r7
     *    are callee-saved under AAPCS, and this handler has no full register-
     *    scrubbing policy. */
    cpsid i

    /* 1. Enable FPU: CP10 + CP11 full access in CPACR (0xE000ED88).
     *    bits[23:20] = 0b1111 => mask 0x00F00000.
     *    Target is thumbv7em-none-eabihf (hard-float); this prevents a silent
     *    NOCP UsageFault on the first VFP instruction. */
    movw r0, #0xED88
    movt r0, #0xE000
    ldr  r1, [r0]
    movw r2, #0x0000
    movt r2, #0x00F0
    orr  r1, r1, r2
    str  r1, [r0]
    dsb  sy
    isb

    /* 3. Zero the ECC-protected L2 SRAM stack window (_sstack .. _estack)
     *    with word stores before moving MSP into it. */
    ldr  r0, ={sstack}
    ldr  r1, ={estack}
    movs r2, #0
1:
    cmp  r0, r1
    bhs  2f
    str  r2, [r0], #4
    b    1b
2:
    /* 4. Set MSP. The HW vector-fetch already loaded _estack into MSP from
     *    BASE_VECTORS[0]; this write is a safety no-op that matches the
     *    production reset handler pattern. */
    mov  sp, r1
    /* 5. Re-enable IRQs after zeroing the L2 SRAM stack and setting MSP.
     *    PRIMASK must be clear before Rust main()/kernel_loop. */
    cpsie i
    /* 6. Hand off to Tock's RAM init (zeroes .bss, copies .data, calls main). */
    ldr  r2, ={init}
    bx   r2
    "#,
        sstack = sym _sstack,
        estack = sym _estack,
        init = sym initialize_ram_jump_to_main,
    );
}

#[cfg(not(any(doc, all(target_arch = "arm", target_os = "none"))))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nxp_s32g3_boot_entry() {}

extern "C" {
    fn _sstack();
    fn _estack();
}
#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    _estack,                      //  0 — initial stack pointer
    nxp_s32g3_boot_entry,         //  1 — Reset (HW loads as PC; Thumb bit automatic)
    unhandled_interrupt,          //  2 — NMI
    CortexM7::HARD_FAULT_HANDLER, //  3 — HardFault
    unhandled_interrupt,          //  4 — MemManage
    unhandled_interrupt,          //  5 — BusFault
    unhandled_interrupt,          //  6 — UsageFault
    unhandled_interrupt,          //  7 — reserved
    unhandled_interrupt,          //  8 — reserved
    unhandled_interrupt,          //  9 — reserved
    unhandled_interrupt,          // 10 — reserved
    CortexM7::SVC_HANDLER,        // 11 — SVCall
    unhandled_interrupt,          // 12 — reserved
    unhandled_interrupt,          // 13 — reserved
    unhandled_interrupt,          // 14 — PendSV
    CortexM7::SYSTICK_HANDLER,
];

#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); mscm::NUM_EXTERNAL_IRQS] =
    [CortexM7::GENERIC_ISR; mscm::NUM_EXTERNAL_IRQS];
