#![no_std]

use cortexm33::{initialize_ram_jump_to_main, unhandled_interrupt, CortexM33, CortexMVariant};

extern "C" {
    fn _estack();
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    _estack,
    initialize_ram_jump_to_main,
    unhandled_interrupt,
    CortexM33::HARD_FAULT_HANDLER,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    CortexM33::SVC_HANDLER,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    CortexM33::SYSTICK_HANDLER,
];
