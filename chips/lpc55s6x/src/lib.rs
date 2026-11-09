// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

#![no_std]
// IOCON has many register definitions in `register_structs()!`
// and requires a deeper recursion limit than the default to fully expand.
#![recursion_limit = "512"]

use cortexm33::{CortexM33, CortexMVariant, initialize_ram_jump_to_main, unhandled_interrupt};

pub mod chip;
pub mod clocks;
pub mod ctimer0;
pub mod flexcomm;
pub mod gpio;
pub mod inputmux;
pub mod interrupts;
pub mod iocon;
pub mod pint;
pub mod pmc;
pub mod rng;
pub mod uart;

extern "C" {
    fn _estack();
}

#[repr(usize)]
#[allow(dead_code)]
enum ImageType {
    Normal = 0x0000,
    PlainSignedImage = 0x0001,
    PlainCRCImage = 0x0002,
    PlainSignedXIPImage = 0x0004,
    PlainCRCXIPImage = 0x0005,
    SignedPlainImageWithKeyStoreIncluded = 0x8001,
}

#[repr(C)]
pub struct Lpc55ImageHeader {
    initial_sp: unsafe extern "C" fn(),
    initial_pc: unsafe extern "C" fn(),
    exception_handlers_2_7: [unsafe extern "C" fn(); 6],
    image_length: usize,
    image_type: ImageType,
    offset_to_specific_header: usize,
    exception_handlers_11_12: [unsafe extern "C" fn(); 2],
    image_execution_address: Option<unsafe extern "C" fn()>,
    exception_handlers_14_15: [unsafe extern "C" fn(); 2],
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
// The reserved Cortex-M33 vector slots at offsets 0x20, 0x24 and 0x28 hold the
// image header that the LPC55 boot ROM validates (UM11126, Table 177). For a
// plain image, imageLength, imageType and offsetToSpecificHeader must be 0;
// otherwise the ROM may, e.g., apply TrustZone preset data that does not exist
// and never start the kernel.
pub static LPC55_IMAGE_HEADER: Lpc55ImageHeader = Lpc55ImageHeader {
    initial_sp: _estack,
    initial_pc: initialize_ram_jump_to_main,
    exception_handlers_2_7: [
        unhandled_interrupt,
        CortexM33::HARD_FAULT_HANDLER,
        unhandled_interrupt,
        unhandled_interrupt,
        unhandled_interrupt,
        unhandled_interrupt,
    ],
    image_length: 0,
    image_type: ImageType::Normal,
    offset_to_specific_header: 0,
    exception_handlers_11_12: [CortexM33::SVC_HANDLER, unhandled_interrupt],
    image_execution_address: Some(unhandled_interrupt),
    exception_handlers_14_15: [unhandled_interrupt, CortexM33::SYSTICK_HANDLER],
};

#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 60] = [
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
    CortexM33::GENERIC_ISR,
];
