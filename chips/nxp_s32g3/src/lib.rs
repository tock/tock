// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![no_std]

pub mod chip;
pub mod linflexd;
pub mod mc_me;
pub mod nvic;
pub mod siul2;
pub mod stm;

use cortexm7::{initialize_ram_jump_to_main, unhandled_interrupt};
use cortexm7::{CortexM7, CortexMVariant};

extern "C" {
    fn _estack();
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    _estack, //  0 — Reset
    initialize_ram_jump_to_main,
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
pub static IRQS: [unsafe extern "C" fn(); nvic::NUM_EXTERNAL_IRQS] =
    [CortexM7::GENERIC_ISR; nvic::NUM_EXTERNAL_IRQS];

pub unsafe fn init() {
    cortexm7::nvic::disable_all();
    cortexm7::nvic::clear_all_pending();
    let vector_table: *const [unsafe extern "C" fn(); 16] = core::ptr::addr_of!(BASE_VECTORS);
    let vector_table: *const () = vector_table.cast();
    cortexm7::scb::set_vector_table_offset(vector_table);
    // MSCM Shared Peripheral Routing: steering LINFlexD interrupts (82, 83, 84) to M7_0 (bit 1)
    for irq in &[82, 83, 84] {
        let addr = (0x4019_8880 + (irq * 2)) as *mut u16;
        core::ptr::write_volatile(addr, core::ptr::read_volatile(addr) | 0x0002);
    }
    cortexm7::nvic::enable_all();
}
