// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Drivers and chip support for ESP32-C3.

#![no_std]

pub mod chip;
pub mod flash;
pub mod i2c;
pub mod intc;
pub mod interrupts;
pub mod rng;
pub mod sysreg;
pub mod usb_serial_jtag;

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
///
/// # Safety
///
/// This is not a function that Rust code must ever call; it is the ESP32-C3
/// chip's post-bootloader entrypoint. It overwrites the `iram` memory segment,
/// and then jumps to `initialize_ram_jump_to_main`, with further overwrites and
/// re-initializes system memory. As such, it would destroy the currently
/// executing Rust program's context.
///
/// ## `naked`
///
/// We use the `naked` attribute to ensure that Rust does not generate a
/// function prologue or epilogue which might reference any data that has not
/// been loaded into or initialized in RAM yet. It does not follow the C calling
/// convention, and must only be executed by the bootloader directly. It will
/// overwrite memory arbitrarily. Rust code must not call this function.
///
/// ## `link_section`
///
/// This function must be located at a specific location in flash memory (at the
/// start of the external SPI flash), following an 8 byte magic string
/// (`0x1d04dbae1d04dbae`). We use the `link_section` directive to place this
/// function into the section called `.esp_direct_boot.entry`. The linker is
/// responsible for inserting the magic string, followed immediately by this
/// function. No other function may be located in this section, to ensure that
/// it is located exactly at the start of the external SPI flash, following the
/// 8 byte magic string.
///
/// ## `no_mangle`
///
/// This function serves as the main entrypoint for ESP32-C3 binaries. To mark
/// it as such in the resulting ELF binary, we export it under the well-known
/// name `esp32_c3_direct_boot_entry`. No other function may be exported under
/// this name.
///
/// Note: this requires a global/uniqueness guarantee.
/// See <https://github.com/tock/tock/issues/5250>.
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".esp_direct_boot.entry")]
#[unsafe(naked)]
pub unsafe extern "C" fn esp32_c3_direct_boot_entry() {
    core::arch::naked_asm!(
        "
    // Disable linker relaxation, as the `gp` is not yet set up.
    .option push
    .option norelax

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

    .option pop
        ",
        start = sym rv32i::initialize_ram_jump_to_main,
    );
}
