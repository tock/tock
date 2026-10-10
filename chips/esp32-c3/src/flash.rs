// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Utilities for the XIP-mapped external SPI flash on the ESP32-C3.

use kernel::debug;
use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::registers::{ReadOnly, register_bitfields, register_structs};

// The register offsets and the bit positions come from
// `components/soc/esp32c3/register/soc/spi_mem_reg.h` in ESP-IDF.
register_structs! {
    SpiMemRegisters {
        (0x000 => _reserved0),
        (0x008 => ctrl: ReadOnly<u32, CTRL::Register>),
        (0x00c => _reserved1),
        (0x014 => clock: ReadOnly<u32, CLOCK::Register>),
        (0x018 => @END),
    }
}

register_bitfields![u32,
    CTRL [
        FASTRD_MODE OFFSET(13) NUMBITS(1) [],
        FREAD_DUAL OFFSET(14) NUMBITS(1) [],
        FREAD_QUAD OFFSET(20) NUMBITS(1) [],
        FREAD_DIO OFFSET(23) NUMBITS(1) [],
        FREAD_QIO OFFSET(24) NUMBITS(1) [],
    ],
    CLOCK [
        CLKCNT_N OFFSET(16) NUMBITS(8) [],
        CLK_EQU_SYSCLK OFFSET(31) NUMBITS(1) [],
    ]
];

/// Prints the read mode and the clock frequency of SPI0 with `debug!`.
///
/// ### Safety
///
/// The caller must run this function on an ESP32-C3. This function reads the
/// SPI0 registers of the ESP32-C3 at 0x6000_3000. On another chip, this address
/// can be unmapped, or belong to a register whose reads have side effects.
pub unsafe fn print_active_flash_mode() {
    // SAFETY:
    //
    // The caller guarantees that this function runs on an ESP32-C3. The SPI0
    // registers of the ESP32-C3 start at 0x6000_3000. Reads of the SPI0 control
    // register and the SPI0 clock register have no side effects.
    let registers: StaticRef<SpiMemRegisters> =
        unsafe { StaticRef::new(0x6000_3000 as *const SpiMemRegisters) };

    let ctrl = registers.ctrl.extract();
    let mode = if ctrl.is_set(CTRL::FREAD_QIO) {
        "QIO"
    } else if ctrl.is_set(CTRL::FREAD_DIO) {
        "DIO"
    } else if ctrl.is_set(CTRL::FREAD_QUAD) {
        "QOUT"
    } else if ctrl.is_set(CTRL::FREAD_DUAL) {
        "DOUT"
    } else if ctrl.is_set(CTRL::FASTRD_MODE) {
        "FAST"
    } else {
        "SLOW"
    };

    // The SPI0 clock source runs at 80 MHz. SPI0 runs at the source frequency
    // when CLK_EQU_SYSCLK is set. SPI0 runs at the source frequency divided by
    // CLKCNT_N + 1 when CLK_EQU_SYSCLK is clear.
    let clock = registers.clock.extract();
    let mhz = if clock.is_set(CLOCK::CLK_EQU_SYSCLK) {
        80
    } else {
        80 / (clock.read(CLOCK::CLKCNT_N) + 1)
    };

    debug!("flash: {} at {} MHz", mode, mhz);
}

/// Switch the SPI flash to QIO or DIO at 80 MHz.
///
/// When the ESP32-C3 boots, its mask ROM leaves the external SPI flash
/// configured in single-bit mode at 20 MHz, which is fairly slow for XIP. This
/// function switches the flash to an 80 MHz clock. It also attempts to use QIO
/// (4-bit per clock cycle) when the QE bit in the SPI flash chip's status
/// register is set, and gracefully falls back to DIO when the QE bit is clear.
/// We never attempt write the QE bit in the flash; a user will need to do that
/// out-of-band.
///
/// Because we're currently already in XIP mode and fetching CPU instructions
/// from the flash (at least when we're hitting a cache-miss), we can't simply
/// muck around with the SPI flash configuration without risk of breaking the
/// current program. Therefore, we place this function into `.iram` (which is an
/// instruction-bus alias of the chip's internal SRAM); it gets re-located by
/// the [`crate::esp32_c3_direct_boot_entry`] assembly routine before any Rust
/// executes. While we're running this function, we must take care to not
/// execute any Rust code (which could reference any arbitrary flash contents).
/// This is calling external functions, but those are located entirely in the
/// chip's ROM and unaffected by the SPI flash re-configuration.
///
/// This function makes these calls in order:
///
/// 1. `esp_rom_spiflash_read_statushigh(chip, &status)` reads the flash status
///    register through SPI1.
/// 2. `esp_rom_spiflash_select_qio_pins(0, 0)` connects the HD and WP pads to
///    SPI0 and SPI1. This call happens only when the QE bit is set.
/// 3. `Wait_SPI_Idle(chip)` waits until SPI0 and SPI1 are idle. It then waits
///    until the write-in-progress bit in the flash status register is clear.
/// 4. `esp_rom_spiflash_config_readmode(mode, 0)` sets the read mode. `mode`
///    is 0 for QIO and 2 for DIO.
/// 5. `esp_rom_spiflash_config_clk(1, 0)` sets the SPI0 clock to 80 MHz.
///
/// The ESP-IDF prototype of `esp_rom_spiflash_config_readmode` is wrong and
/// missing the second argument. The mask ROM clears the QE bit by writing to
/// the SPI flash when the second argument is non-zero.
///
/// ### Safety
///
/// The caller must uphold these conditions:
///
/// 1. This function is run only on an ESP32-C3 with a compatible ROM that
///    places the following symbols and data at the corresponding addresses (see
///    the implementation's assembly clobbers for more details):
///
///    - 0x4000015c: function esp_rom_spiflash_read_statushigh
///    - 0x4000013c: function esp_rom_spiflash_select_qio_pins
///    - 0x40000154: function esp_rom_spiflash_config_readmode
///    - 0x40000150: function esp_rom_spiflash_config_clk
///    - 0x4000021c: function Wait_SPI_Idle
///    - 0x3fcdfff0: struct rom_spiflash_legacy_data
///
/// 2. The chip's entry stub ([`crate::esp32_c3_direct_boot_entry`]) has been
///    run, and it has copied the `.iram` section into the `iram` region. No
///    code has written to the `iram` region (e.g., through its data bus alias)
///    since then.
///
/// 3. The CPU takes no trap during the call because the trap handlers execute
///    from flash. The caller disables interrupts by clearing `mstatus.MIE` to
///    meet this condition.
///
/// 4. No code has written to the SRAM locations that the mask ROM reserves.
///    The mask ROM reserves all SRAM memory starting at 0x3FCDF000.
///
/// 5. The external SPI flash module uses bit 9 of its status register as the QE
///    bit. The flash supports QIO (if QE is set) and DIO at 80 MHz.
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[link_section = ".iram.esp32_c3_flash_configure"]
#[unsafe(naked)]
pub unsafe extern "C" fn configure() {
    core::arch::naked_asm!(
        "
    addi sp, sp, -16
    sw   ra, 12(sp)
    sw   s0, 8(sp)
    sw   s1, 4(sp)

    // Load the address of the `esp_rom_spiflash_chip_t` SPI flash descriptor
    li   t0, {rom_spiflash_legacy_data}
    lw   s0, 0(t0)              // s0 = `esp_rom_spiflash_chip_t*`

    // Read the status of the SPI flash chip, to determine if the QE bit is
    // enabled and we can thus switch to QIO (4 bit per clock cycle).
    mv   a0, s0                 // a0 = s0 = &spi (1st param)
    mv   a1, sp                 // a1 = &status (2nd param)
    li   t0, {esp_rom_spiflash_read_statushigh}
    jalr t0                     // esp_rom_spiflash_read_statushigh(spi, status)

    // Take this branch of status read failed; we'll stick to DIO at 80 MHz.
    li   s1, 2                  // s1 = mode = DIO
    bnez a0, 100f               // Keep DIO if the status read failed.

    // Status read succeeded, check the QE bit.
    lw   t1, 0(sp)              // t1 = *status (written by `_statushigh`)
    andi t1, t1, 0x200          // t1 &= 1 << 9 (QE bit)

    // Take this branch if !QE, in which case we'll stick to DIO at 80 MHz.
    beqz t1, 100f               // Keep DIO if the QE bit is clear.

    // QE is set, let's switch to QIO. Configure the GPIOs for the two addl.
    // data lines first.
    li   a0, 0                  // a0 = wp_gpio_num = 0
    li   a1, 0                  // a1 = spiconfig = 0
    li   t0, {esp_rom_spiflash_select_qio_pins}
    jalr t0                     // esp_rom_spiflash_select_qio_pins(0, 0)
    li   s1, 0                  // s1 = mode = QIO

100:
    // Now, configure the actual SPI mode, and SPI clock. `s1` indicates
    // whether we'll be using DIO (s1 == 2) or QIO (s1 == 0). Before we do this,
    // wait for the SPI bus towards the flash chip to be idle. This code runs in
    // SRAM and so won't use the flash.
    mv   a0, s0                 // a0 = s0 = `esp_rom_spiflash_chip_t*`
    li   t0, {Wait_SPI_Idle}
    jalr t0                     // Wait_SPI_Idle(&chip)

    mv   a0, s1                 // a0 = s1 = DIO (2) | QIO (0)
    li   a1, 0                  // a1 = 0
    li   t0, {esp_rom_spiflash_config_readmode}
    jalr t0                     // esp_rom_spiflash_config_readmode(a0, 0)

    li   a0, 1                  // a0 = 1
    li   a1, 0                  // a1 = 0
    li   t0, {esp_rom_spiflash_config_clk}
    jalr t0                     // esp_rom_spiflash_config_clk(1, 0)

    // Restore clobbered variables, and return. We can now use the reconfigured
    // SPI flash and return to XIP code.
    lw   ra, 12(sp)
    lw   s0, 8(sp)
    lw   s1, 4(sp)
    addi sp, sp, 16
    ret
        ",
        // Addrs from `components/esp_rom/esp32c3/ld/esp32c3.rom.ld` in ESP-IDF.
        //
        // `components/esp_rom/esp32c3/include/esp32c3/rom/spi_flash.h`:
        // esp_rom_spiflash_result_t esp_rom_spiflash_read_statushigh(
        //     esp_rom_spiflash_chip_t *spi, uint32_t *status);
        esp_rom_spiflash_read_statushigh = const 0x4000015c,
        // `components/esp_rom/esp32c3/include/esp32c3/rom/spi_flash.h`:
        // void esp_rom_spiflash_select_qio_pins(
        //     uint8_t wp_gpio_num, uint32_t spiconfig);
        esp_rom_spiflash_select_qio_pins = const 0x4000013c,
        // `components/esp_rom/esp32c3/include/esp32c3/rom/spi_flash.h`:
        // esp_rom_spiflash_result_t esp_rom_spiflash_config_readmode(
        //     esp_rom_spiflash_read_mode_t mode, uint32_t a1);
        //
        // The ESP-IDF prototype is wrong, and omits the second parameter. When
        // `a1` is non-zero and mode isn't quad, the ROM will clear the flash's
        // QE bit (actually writes to the SPI flash).
        esp_rom_spiflash_config_readmode = const 0x40000154,
        // esp_rom_spiflash_result_t esp_rom_spiflash_config_clk(
        //     uint8_t freqdiv, uint8_t spi);
        //
        // The signature comes from
        // `components/esp_rom/esp32c3/include/esp32c3/rom/spi_flash.h` in ESP-IDF.
        esp_rom_spiflash_config_clk = const 0x40000150,
        // esp_rom_spiflash_result_t Wait_SPI_Idle(esp_rom_spiflash_chip_t *spi);
        //
        // ESP-IDF does not declare this function; it's an internal helper of
        // the mask ROM. It waits until SPI0 and SPI1 are idle, and then waits
        // until the write-in-progress bit of the flash status reg is clear.
        Wait_SPI_Idle = const 0x4000021c,
        // Static global of the mask ROM. It holds a pointer to the flash state
        // of the mask ROM. The flash state starts with the
        // `esp_rom_spiflash_chip_t` SPI flash chip descriptor.
        //
        // The layout of the flash state comes from a disassembly of the mask
        // ROM. ESP-IDF does not declare this variable.
        rom_spiflash_legacy_data = const 0x3fcdfff0,
    );
}

// Mock implementation for static checks and CI.
#[cfg(not(all(target_arch = "riscv32", target_os = "none")))]
pub unsafe extern "C" fn configure() {
    unimplemented!()
}
