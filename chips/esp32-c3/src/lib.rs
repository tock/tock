// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Drivers and chip support for ESP32-C3.

#![no_std]

pub mod chip;
pub mod flash;
pub mod intc;
pub mod interrupts;
pub mod rng;
pub mod sysreg;

pub mod timg {
    pub use esp32::timg::{ClockSource, TIMG0_BASE, TIMG1_BASE};
    pub type TimG<'a> = esp32::timg::TimG<'a, esp32::timg::Freq20MHz, true>;
}

/// Entry point of the kernel.
///
/// The mask ROM jumps to this function in direct boot mode. `layout.ld` places
/// this function directly after the magic words at the start of flash.
///
/// This function copies the `.iram` section from flash into SRAM. It then
/// jumps to `initialize_ram_jump_to_main`, which is the RISC-V `_start` of
/// Tock. The copy runs before any Rust code because Rust code can call
/// functions in the `.iram` section.
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[no_mangle]
#[link_section = ".esp_direct_boot.entry"]
#[unsafe(naked)]
pub unsafe extern "C" fn esp32_c3_direct_boot_entry() {
    core::arch::naked_asm!(
        "
    la a0, _siram_dbus          // a0 = data-bus address of the start of .iram
    la a1, _eiram_dbus          // a1 = data-bus address of the end of .iram
    la a2, _siram_load          // a2 = flash address of the start of .iram

100:
    beq  a0, a1, 101f           // start ?= end -> 101f
    lw   a3, 0(a2)              // load word from flash
    sw   a3, 0(a0)              // write word to data-bus alias of SRAM
    addi a0, a0, 4              // dbus_addr += 4
    addi a2, a2, 4              // flash_addr += 4
    j 100b

101:
    fence.i                     // Make sure the CPU sees the code
    j {start}                   // Run the main RISC-V start code
        ",
        start = sym rv32i::initialize_ram_jump_to_main,
    );
}
