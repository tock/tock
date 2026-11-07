// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Support for driving the external, XIP-mapped SPI flash of the ESP32-C3.
//!
//! This module serves two purposes: it provides [`SpiFlash`], which is a
//! [`hil::flash::Flash`] driver for the external SPI flash attached to an
//! ESP32-C3. It expects the flash to be linearly mapped into the address space
//! by the ESP32-C3's bootloader / mask ROM starting at address 0x4200_0000 (the
//! instruction-bus alias, which also supports reads), and issues erases and
//! writes to the flash through support routines baked into the mask ROM of the
//! ESP32-C3 chip.
//!
//! Next to this, it also provides routines that can modify the SPI flash clock
//! and switch it from DIO to QIO (4 bits per clock cycle) mode.
//!
//! Most such routines in this module are necessarily implemented using
//! assembly: writing to the SPI flash, or changing its clock or protocol
//! configuration prevents us from executing from it at the same time (which
//! would yield garbage). We work around that by implementing these functions in
//! assembly that's loaded into a region of the chip's SRAM on boot (we call
//! this IRAM, as we need to execute it through an instruction bus alias). These
//! functions only reference code or data from either SRAM or the chip's mask
//! ROM, and never the SPI flash. They return control to the Rust caller only
//! once XIP works again.

use core::cell::Cell;

use kernel::ErrorCode;
use kernel::debug;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::registers::{ReadOnly, register_bitfields, register_structs};

/// Size of a [`SpiFlash`] page, which is one erasable sector of the flash.
pub const PAGE_SIZE: usize = 4096;

/// Size of a flash page that [`write`](fn@write) programs at once.
const PROGRAM_SIZE: usize = 256;

/// Address at which the mask ROM's direct boot mode maps the start of the
/// external SPI flash on the instruction bus.
pub const XIP_BASE: usize = 0x4200_0000;

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
    // SAFETY: The caller guarantees that this function runs on an ESP32-C3. The
    // SPI0 registers of the ESP32-C3 start at 0x6000_3000. Reads of the SPI0
    // control register and the SPI0 clock register have no side effects.
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
/// # Safety
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
///
/// ## `naked`
///
/// The assembly of this function must execute from IRAM, as it reconfigures the
/// way that the ESP32-C3 accesses its XIP flash. A regular function containing
/// an `asm!` block does not guarantee this: the compiler may inline or outline
/// it into code that executes from flash, which would defeat its
/// `link_section`. A naked function is emitted as exactly its assembly, under
/// its own symbol, and is never inlined. It follows the standard C calling
/// convention and can be invoked directly from Rust. It returns once the SPI
/// flash can be safely accessed again.
///
/// ## `link_section`
///
/// This function must be located entirely in SRAM, not reference the external
/// SPI flash, and be executed from the instruction bus alias of SRAM memory
/// (IRAM). We use the `link_section` attribute to place this function into a
/// `.iram.esp32_c3_flash_configure`. The linker script is responsible for
/// ensuring that this function is located in IRAM. Callers must ensure that
/// IRAM is properly boostrapped from flash prior to calling this function.
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[unsafe(link_section = ".iram.esp32_c3_flash_configure")]
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

    mv   a0, s1                 // a0 = s1 = mode (DIO=2 or QIO=0)
    li   a1, 0                  // a1 = 0
    li   t0, {esp_rom_spiflash_config_readmode}
    jalr t0                     // esp_rom_spiflash_config_readmode(mode, 0)

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

/// Mock implementation for static checks and CI.
///
/// # Safety
///
/// Unused mock implementation.
#[cfg(not(all(target_arch = "riscv32", target_os = "none")))]
pub unsafe extern "C" fn configure() {
    unimplemented!()
}

/// Erase one 4 KiB sector of the SPI flash.
///
/// The CPU fetches instructions from the external SPI flash through the cache
/// (XIP). While the flash erases a sector, it does not answer reads, and a
/// cache miss would fetch garbage. Therefore, we place this function into
/// `.iram` (which is an instruction-bus alias of the chip's internal SRAM); it
/// gets re-located by the [`crate::esp32_c3_direct_boot_entry`] assembly
/// routine before any Rust executes. It suspends the cache for the duration of
/// the erase. While we're running this function, we must take care to not
/// execute any Rust code (which could reference any arbitrary flash contents).
/// This is calling external functions, but those are located entirely in the
/// chip's mask ROM and unaffected by the SPI flash being busy.
///
/// This function makes these calls in order:
///
/// 1. `Cache_Suspend_ICache()` waits until the cache is idle, suspends it, and
///    returns whether autoload (Espressif's term for the instruction cache's
///    hardware prefetching) was enabled, so that step 3 can restore it.
///
/// 2. `esp_rom_spiflash_erase_sector(sector)` erases the sector through SPI1.
///    It returns once the write-in-progress bit in the flash status register is
///    clear.
///
/// 3. `Cache_Resume_ICache(autoload)` resumes the cache.
///
/// 4. `Cache_Invalidate_Addr(0x4200_0000 + sector * 4096, 4096)` invalidates
///    the cache lines that map the erased sector, so that subsequent reads
///    through the XIP mapping observe the erased contents.
///
/// It clears `mstatus.MIE` before the first call, and restores it after the
/// last call (if it was previously enabled). This ensures we don't execute any
/// code of the trap handler in XIP flash.
///
/// It returns the `esp_rom_spiflash_result_t` of the erase, which is 0 on
/// success, 1 on an error, and 2 on a timeout.
///
/// # Safety
///
/// The caller must uphold these conditions:
///
/// 1. This function is run only on an ESP32-C3 with a compatible ROM that
///    places the following symbols at the corresponding addresses (see the
///    implementation's assembly clobbers for more details):
///
///    - 0x40000524: function Cache_Suspend_ICache
///    - 0x40000128: function esp_rom_spiflash_erase_sector
///    - 0x40000528: function Cache_Resume_ICache
///    - 0x400004d4: function Cache_Invalidate_Addr
///
/// 2. The chip's entry stub ([`crate::esp32_c3_direct_boot_entry`]) has been
///    run, and it has copied the `.iram` section into the `iram` region. No
///    code has written to the `iram` region (e.g., through its data bus alias)
///    since then.
///
/// 3. The CPU runs in machine mode. This function clears `mstatus.MIE` to take
///    no trap during the call, because the trap handlers execute from flash.
///
/// 4. No code has written to the SRAM locations that the mask ROM reserves.
///    The mask ROM reserves all SRAM memory starting at 0x3FCDF000.
///
/// 5. The mask ROM maps the external SPI flash linearly at 0x4200_0000, and
///    its flash descriptor (`esp_rom_spiflash_chip_t`) uses 4 KiB sectors and
///    covers the sector (see [`SpiFlash::new`]).
///
/// 6. The sector holds no code or data that the kernel or any process depends
///    on.
///
/// ## `naked`
///
/// The assembly of this function must execute from IRAM, as it suspends the
/// cache through which the ESP32-C3 accesses its XIP flash. A regular function
/// containing an `asm!` block does not guarantee this: the compiler may inline
/// or outline it into code that executes from flash, which would defeat its
/// `link_section`. A naked function is emitted as exactly its assembly, under
/// its own symbol, and is never inlined. It follows the standard C calling
/// convention and can be invoked directly from Rust. It returns once the SPI
/// flash can be safely accessed again.
///
/// ## `link_section`
///
/// This function must be located entirely in SRAM, not reference the external
/// SPI flash, and be executed from the instruction bus alias of SRAM memory
/// (IRAM). We use the `link_section` attribute to place this function into a
/// `.iram.esp32_c3_flash_erase_sector`. The linker script is responsible for
/// ensuring that this function is located in IRAM. Callers must ensure that
/// IRAM is properly boostrapped from flash prior to calling this function.
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[unsafe(link_section = ".iram.esp32_c3_flash_erase_sector")]
#[unsafe(naked)]
pub unsafe extern "C" fn erase_sector(sector: u32) -> u32 {
    core::arch::naked_asm!(
        "
    addi sp, sp, -32
    sw   ra, 28(sp)
    sw   s0, 24(sp)
    sw   s1, 20(sp)
    sw   s2, 16(sp)
    sw   s3, 12(sp)

    mv   s0, a0                 // s0 = sector
    csrrci s1, mstatus, 0x8     // s1 = mstatus, then clear mstatus.MIE

    // Suspend the cache, so that no access reaches the flash while it erases
    // the sector. This code runs in SRAM and so won't use the flash.
    li   t0, {Cache_Suspend_ICache}
    jalr t0                     // a0 = Cache_Suspend_ICache()
    mv   s2, a0                 // s2 = autoload

    mv   a0, s0                 // a0 = s0 = sector
    li   t0, {esp_rom_spiflash_erase_sector}
    jalr t0                     // a0 = esp_rom_spiflash_erase_sector(sector)
    mv   s3, a0                 // s3 = result

    mv   a0, s2                 // a0 = s2 = autoload
    li   t0, {Cache_Resume_ICache}
    jalr t0                     // Cache_Resume_ICache(autoload)

    // Drop the cache lines that hold the old contents of the sector.
    slli a0, s0, 12             // a0 = sector * 4096
    li   t0, {xip_base}
    add  a0, a0, t0             // a0 = 0x42000000 + sector * 4096
    li   a1, 4096               // a1 = 4096
    li   t0, {Cache_Invalidate_Addr}
    jalr t0                     // Cache_Invalidate_Addr(vaddr, 4096)

    // Restore mstatus.MIE, and return the result of the erase. We can now use
    // the SPI flash and return to XIP code.
    andi s1, s1, 0x8            // s1 = previous mstatus.MIE
    csrs mstatus, s1            // set mstatus.MIE if it was set
    mv   a0, s3                 // a0 = s3 = result
    lw   ra, 28(sp)
    lw   s0, 24(sp)
    lw   s1, 20(sp)
    lw   s2, 16(sp)
    lw   s3, 12(sp)
    addi sp, sp, 32
    ret
        ",
        // Addrs from `components/esp_rom/esp32c3/ld/esp32c3.rom.ld` in ESP-IDF.
        //
        // `components/esp_rom/esp32c3/include/esp32c3/rom/cache.h`:
        // uint32_t Cache_Suspend_ICache(void);
        Cache_Suspend_ICache = const 0x40000524,
        // `components/esp_rom/esp32c3/include/esp32c3/rom/spi_flash.h`:
        // esp_rom_spiflash_result_t esp_rom_spiflash_erase_sector(
        //     uint32_t sector_num);
        esp_rom_spiflash_erase_sector = const 0x40000128,
        // `components/esp_rom/esp32c3/include/esp32c3/rom/cache.h`:
        // void Cache_Resume_ICache(uint32_t autoload);
        Cache_Resume_ICache = const 0x40000528,
        // `components/esp_rom/esp32c3/include/esp32c3/rom/cache.h`:
        // int Cache_Invalidate_Addr(uint32_t addr, uint32_t size);
        Cache_Invalidate_Addr = const 0x400004d4,
        xip_base = const XIP_BASE,
    );
}

/// Mock implementation for static checks and CI.
///
/// # Safety
///
/// Unused mock implementation.
#[cfg(not(all(target_arch = "riscv32", target_os = "none")))]
pub unsafe extern "C" fn erase_sector(_sector: u32) -> u32 {
    unimplemented!()
}

/// Write to one 256-byte page of the SPI flash.
///
/// The CPU fetches instructions from the external SPI flash through the cache
/// (XIP). While the flash programs a page, it does not answer reads, and a
/// cache miss would fetch garbage. Therefore, we place this function into
/// `.iram` (which is an instruction-bus alias of the chip's internal SRAM); it
/// gets re-located by the [`crate::esp32_c3_direct_boot_entry`] assembly
/// routine before any Rust executes. It suspends the cache for the duration of
/// the write. While we're running this function, we must take care to not
/// execute any Rust code (which could reference any arbitrary flash contents).
/// This is calling external functions, but those are located entirely in the
/// chip's mask ROM and unaffected by the SPI flash being busy.
///
/// This function makes these calls in order:
///
/// 1. `Cache_Suspend_ICache()` waits until the cache is idle, suspends it, and
///    returns whether autoload (Espressif's term for the instruction cache's
///    hardware prefetching) was enabled, so that step 3 can restore it.
///
/// 2. `esp_rom_spiflash_write(offset, src, len)` writes `len` bytes from `src`
///    to the flash at `offset` through SPI1. It returns once the
///    write-in-progress bit in the flash status register is clear.
///
/// 3. `Cache_Resume_ICache(autoload)` resumes the cache.
///
/// 4. `Cache_Invalidate_Addr(0x4200_0000 + offset, len)` invalidates the cache
///    lines that map the written bytes, so that subsequent reads through the
///    XIP mapping observe the written contents.
///
/// It clears `mstatus.MIE` before the first call, and restores it after the
/// last call (if it was previously enabled). This ensures we don't execute any
/// code of the trap handler in XIP flash.
///
/// It returns the `esp_rom_spiflash_result_t` of the write, which is 0 on
/// success, 1 on an error, and 2 on a timeout.
///
/// Like any NOR flash, the flash can only clear bits when it writes. A write
/// to bytes that have not been erased stores the bitwise AND of their old and
/// new contents.
///
/// # Safety
///
/// The caller must uphold these conditions:
///
/// 1. This function is run only on an ESP32-C3 with a compatible ROM that
///    places the following symbols at the corresponding addresses (see the
///    implementation's assembly clobbers for more details):
///
///    - 0x40000524: function Cache_Suspend_ICache
///    - 0x4000012c: function esp_rom_spiflash_write
///    - 0x40000528: function Cache_Resume_ICache
///    - 0x400004d4: function Cache_Invalidate_Addr
///
/// 2. The chip's entry stub ([`crate::esp32_c3_direct_boot_entry`]) has been
///    run, and it has copied the `.iram` section into the `iram` region. No
///    code has written to the `iram` region (e.g., through its data bus alias)
///    since then.
///
/// 3. The CPU runs in machine mode. This function clears `mstatus.MIE` to take
///    no trap during the call, because the trap handlers execute from flash.
///
/// 4. No code has written to the SRAM locations that the mask ROM reserves.
///    The mask ROM reserves all SRAM memory starting at 0x3FCDF000.
///
/// 5. The mask ROM maps the external SPI flash linearly at 0x4200_0000, and
///    its flash descriptor (`esp_rom_spiflash_chip_t`) covers the written
///    bytes (see [`SpiFlash::new`]).
///
/// 6. `offset` and `len` are multiples of 4, and the bytes `offset..offset +
///    len` lie within one 256-byte page of the flash. These bytes hold no code
///    or data that the kernel or any process depends on.
///
/// 7. `src` is 4-byte aligned and points to `len` bytes in SRAM, which are
///    valid for reads for the duration of the call. They must not be in the
///    flash, which the CPU cannot read while the cache is suspended.
///
/// ## `naked`
///
/// The assembly of this function must execute from IRAM, as it suspends the
/// cache through which the ESP32-C3 accesses its XIP flash. A regular function
/// containing an `asm!` block does not guarantee this: the compiler may inline
/// or outline it into code that executes from flash, which would defeat its
/// `link_section`. A naked function is emitted as exactly its assembly, under
/// its own symbol, and is never inlined. It follows the standard C calling
/// convention and can be invoked directly from Rust. It returns once the SPI
/// flash can be safely accessed again.
///
/// ## `link_section`
///
/// This function must be located entirely in SRAM, not reference the external
/// SPI flash, and be executed from the instruction bus alias of SRAM memory
/// (IRAM). We use the `link_section` attribute to place this function into a
/// `.iram.esp32_c3_flash_write`. The linker script is responsible for ensuring
/// that this function is located in IRAM. Callers must ensure that IRAM is
/// properly boostrapped from flash prior to calling this function.
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[unsafe(link_section = ".iram.esp32_c3_flash_write")]
#[unsafe(naked)]
pub unsafe extern "C" fn write(offset: u32, src: *const u32, len: u32) -> u32 {
    core::arch::naked_asm!(
        "
    addi sp, sp, -32
    sw   ra, 28(sp)
    sw   s0, 24(sp)
    sw   s1, 20(sp)
    sw   s2, 16(sp)
    sw   s3, 12(sp)
    sw   s4, 8(sp)

    mv   s0, a0                 // s0 = offset
    mv   s3, a1                 // s3 = src
    mv   s4, a2                 // s4 = len
    csrrci s1, mstatus, 0x8     // s1 = mstatus, then clear mstatus.MIE

    // Suspend the cache, so that no access reaches the flash while it programs
    // the page. This code runs in SRAM and so won't use the flash.
    li   t0, {Cache_Suspend_ICache}
    jalr t0                     // a0 = Cache_Suspend_ICache()
    mv   s2, a0                 // s2 = autoload

    mv   a0, s0                 // a0 = s0 = offset
    mv   a1, s3                 // a1 = s3 = src
    mv   a2, s4                 // a2 = s4 = len
    li   t0, {esp_rom_spiflash_write}
    jalr t0                     // a0 = esp_rom_spiflash_write(offset, src, len)
    mv   s3, a0                 // s3 = result

    mv   a0, s2                 // a0 = s2 = autoload
    li   t0, {Cache_Resume_ICache}
    jalr t0                     // Cache_Resume_ICache(autoload)

    // Drop the cache lines that hold the old contents of the written bytes.
    li   t0, {xip_base}
    add  a0, s0, t0             // a0 = 0x42000000 + offset
    mv   a1, s4                 // a1 = s4 = len
    li   t0, {Cache_Invalidate_Addr}
    jalr t0                     // Cache_Invalidate_Addr(vaddr, len)

    // Restore mstatus.MIE, and return the result of the write. We can now use
    // the SPI flash and return to XIP code.
    andi s1, s1, 0x8            // s1 = previous mstatus.MIE
    csrs mstatus, s1            // set mstatus.MIE if it was set
    mv   a0, s3                 // a0 = s3 = result
    lw   ra, 28(sp)
    lw   s0, 24(sp)
    lw   s1, 20(sp)
    lw   s2, 16(sp)
    lw   s3, 12(sp)
    lw   s4, 8(sp)
    addi sp, sp, 32
    ret
        ",
        // Addrs from `components/esp_rom/esp32c3/ld/esp32c3.rom.ld` in ESP-IDF.
        //
        // `components/esp_rom/esp32c3/include/esp32c3/rom/cache.h`:
        // uint32_t Cache_Suspend_ICache(void);
        Cache_Suspend_ICache = const 0x40000524,
        // `components/esp_rom/esp32c3/include/esp32c3/rom/spi_flash.h`:
        // esp_rom_spiflash_result_t esp_rom_spiflash_write(
        //     uint32_t dest_addr, const uint32_t *src, int32_t len);
        esp_rom_spiflash_write = const 0x4000012c,
        // `components/esp_rom/esp32c3/include/esp32c3/rom/cache.h`:
        // void Cache_Resume_ICache(uint32_t autoload);
        Cache_Resume_ICache = const 0x40000528,
        // `components/esp_rom/esp32c3/include/esp32c3/rom/cache.h`:
        // int Cache_Invalidate_Addr(uint32_t addr, uint32_t size);
        Cache_Invalidate_Addr = const 0x400004d4,
        xip_base = const XIP_BASE,
    );
}

/// Mock implementation for static checks and CI.
///
/// # Safety
///
/// Unused mock implementation.
#[cfg(not(all(target_arch = "riscv32", target_os = "none")))]
pub unsafe extern "C" fn write(_offset: u32, _src: *const u32, _len: u32) -> u32 {
    unimplemented!()
}

/// Sets the size in the mask ROM's flash descriptor to at least `size` bytes,
/// with 4 KiB sectors.
///
/// The mask ROM's erase and write routines reject operations beyond this size.
/// Tock uses the ESP32-C3 ROM's direct boot mode, where no other code sets this
/// value.
///
/// # Safety
///
/// This function is run only on an ESP32-C3 with a compatible ROM that places
/// `esp_rom_spiflash_config_param` at 0x40000134, and stores a pointer to its
/// flash descriptor at 0x3fcdfff0. The flash holds at least `size` bytes.
unsafe fn set_chip_size(size: u32) {
    // SAFETY:
    //
    // The caller guarantees that 0x3fcdfff0 holds a pointer to the mask ROM's
    // flash state, which starts with the `esp_rom_spiflash_chip_t` descriptor
    // (as in `configure`). The descriptor's first four `uint32_t` fields are
    // `device_id`, `chip_size`, `block_size` and `sector_size`
    // (`components/esp_rom/esp32c3/include/esp32c3/rom/spi_flash.h`). The ROM
    // does not modify the descriptor concurrently: the ROM's flash routines
    // only run when the kernel calls them. The pointer and the fields are
    // aligned `uint32_t`s in SRAM, whose reads have no side effects.
    let (device_id, chip_size, sector_size) = unsafe {
        let chip = *(0x3fcd_fff0 as *const *const u32);
        (*chip, *chip.add(1), *chip.add(3))
    };

    if chip_size >= size && sector_size == PAGE_SIZE as u32 {
        return;
    }

    // We call the ROM function through `asm!` rather than by transmuting its
    // address to a function pointer, as Rust's rules for integer-to-pointer
    // provenance are still being worked out.
    //
    // SAFETY:
    //
    // The caller guarantees that `esp_rom_spiflash_config_param` is at
    // 0x40000134 and the flash holds `size` bytes. The function only stores its
    // arguments in the descriptor; it does not access the flash, so it can run
    // from XIP code. It follows the C calling convention, and `clobber_abi("C")`
    // covers the registers it may clobber. The block, sector and page sizes are
    // those that ESP-IDF's bootloader passes for all supported flash chips.
    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    unsafe {
        core::arch::asm!(
            "
            li   t0, {esp_rom_spiflash_config_param}
            jalr t0
            ",
            // `components/esp_rom/esp32c3/include/esp32c3/rom/spi_flash.h`:
            // esp_rom_spiflash_result_t esp_rom_spiflash_config_param(
            //     uint32_t deviceId, uint32_t chip_size, uint32_t block_size,
            //     uint32_t sector_size, uint32_t page_size,
            //     uint32_t status_mask);
            esp_rom_spiflash_config_param = const 0x40000134,
            in("a0") device_id,
            in("a1") size,
            in("a2") 0x10000,
            in("a3") PAGE_SIZE as u32,
            in("a4") 0x100,
            in("a5") 0xffff,
            clobber_abi("C"),
        );
    }

    // Mock implementation for static checks and CI.
    #[cfg(not(all(target_arch = "riscv32", target_os = "none")))]
    {
        let _ = device_id;
        unimplemented!()
    }
}

/// A page of the SPI flash, aligned to 4 bytes as [`write`](fn@write)
/// requires.
#[repr(C, align(4))]
pub struct Esp32C3Page(pub [u8; PAGE_SIZE]);

impl Default for Esp32C3Page {
    fn default() -> Self {
        Self([0; PAGE_SIZE])
    }
}

impl AsMut<[u8]> for Esp32C3Page {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Operation whose completion is pending.
#[derive(Clone, Copy)]
enum Operation {
    Read,
    Write,
    Erase,
}

/// [`hil::flash::Flash`] driver for the external SPI flash, with one page per
/// 4 KiB sector.
///
/// Reads copy from the XIP mapping. Writes and erases call the mask ROM's SPI
/// flash routines through [`write`](fn@write) and [`erase_sector`].
///
/// Writes and erases block with interrupts disabled: a write for the program
/// time of each 256-byte chunk that is not all `0xFF`, and an erase for the
/// erase time of the whole sector.
pub struct SpiFlash {
    /// Size of the flash in bytes.
    size: usize,
    client: OptionalCell<&'static dyn hil::flash::Client<SpiFlash>>,
    buffer: TakeCell<'static, Esp32C3Page>,
    pending: Cell<Option<(Operation, Result<(), hil::flash::Error>)>>,
    deferred_call: DeferredCall,
}

impl SpiFlash {
    /// Create a driver for a flash of `size` bytes.
    ///
    /// The mask ROM's erase and write routines bound and address their
    /// operations using the chip size and sector size in the ROM's flash
    /// descriptor. In direct boot mode, no second-stage bootloader configures
    /// this descriptor, so this function sets it to `size` bytes with 4 KiB
    /// sectors if it lists less than `size` bytes or a different sector size.
    /// The descriptor is global state of the mask ROM, shared by all of its SPI
    /// flash routines.
    ///
    /// # Safety
    ///
    /// The caller must uphold these conditions:
    ///
    /// 1. `size` is a multiple of [`PAGE_SIZE`], and the external SPI flash
    ///    holds at least `size` bytes, all of which the mask ROM maps linearly
    ///    at 0x4200_0000.
    ///
    /// 2. The safety conditions of [`erase_sector`] and [`write`](fn@write)
    ///    other than those on their arguments hold whenever the driver is used.
    ///    In particular, the ROM addresses of these functions and of
    ///    `esp_rom_spiflash_config_param` (0x40000134) and
    ///    `rom_spiflash_legacy_data` (0x3fcdfff0) are correct.
    ///
    /// 3. Only one `SpiFlash` exists.
    ///
    /// 4. The driver's users only write and erase pages that hold no code or
    ///    data that the kernel or processes depend on.
    pub unsafe fn new(size: usize) -> Self {
        // SAFETY: The caller guarantees that this runs on an ESP32-C3 with the
        // expected mask ROM, and that the flash holds `size` bytes.
        unsafe { set_chip_size(size as u32) };

        Self {
            size,
            client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            pending: Cell::new(None),
            deferred_call: DeferredCall::new(),
        }
    }

    /// Returns the flash offset of `page_number`, if it is on the flash.
    fn offset(&self, page_number: usize) -> Option<usize> {
        let offset = page_number.checked_mul(PAGE_SIZE)?;
        (offset < self.size).then_some(offset)
    }

    /// Records `operation` as complete, and schedules its callback.
    fn complete(&self, operation: Operation, result: Result<(), hil::flash::Error>) {
        self.pending.set(Some((operation, result)));
        self.deferred_call.set();
    }
}

impl<C: hil::flash::Client<Self>> hil::flash::HasClient<'static, C> for SpiFlash {
    fn set_client(&self, client: &'static C) {
        self.client.set(client);
    }
}

impl hil::flash::Flash for SpiFlash {
    type Page = Esp32C3Page;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        if self.pending.get().is_some() {
            return Err((ErrorCode::BUSY, buf));
        }
        let Some(offset) = self.offset(page_number) else {
            return Err((ErrorCode::INVAL, buf));
        };

        for (i, word) in buf.0.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            // SAFETY: `new`'s caller guarantees that the mask ROM maps the
            // flash linearly at `XIP_BASE`, and `offset` lies on the flash. The
            // read is aligned, and has no side effects. The flash only changes
            // within `erase_sector` and `write`, which do not run concurrently
            // with this read. The read is volatile because the XIP mapping is
            // memory that is not (entirely) part of a Rust allocation, which
            // Rust permits only volatile accesses to.
            let value =
                unsafe { core::ptr::read_volatile((XIP_BASE + offset + i * 4) as *const u32) };
            *word = value.to_le_bytes();
        }

        self.buffer.replace(buf);

        // This schedules a deferred call, as we can't synchronously invoke the
        // callback here:
        self.complete(Operation::Read, Ok(()));

        Ok(())
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        if self.pending.get().is_some() {
            return Err((ErrorCode::BUSY, buf));
        }

        let Some(offset) = self.offset(page_number) else {
            return Err((ErrorCode::INVAL, buf));
        };

        // Program the page in chunks of 256 bytes. Writing `0xFF` leaves NOR
        // flash unchanged, so we skip those chunks. Interrupts are enabled in
        // between chunks (right now, this doesn't matter because the kernel
        // can't schedule anything else while executing this function, but it
        // provisions for us later having some truly-async timing critical tasks
        // serviced in between these expensive flash writes).
        let mut result = Ok(());
        for (i, chunk) in buf.0.as_chunks::<PROGRAM_SIZE>().0.iter().enumerate() {
            if chunk.iter().all(|&b| b == 0xFF) {
                continue;
            }

            // SAFETY: This upholds the safety conditions of `write`:
            //
            // - ROM addresses, `.iram` contents, machine mode and the ROM's
            //   reserved SRAM are discharged by the SpiFlash::new constructor.
            //
            // - Flash mapping and descriptor: `SpiFlash::new`'s caller
            //   guarantees that the mask ROM maps the flash linearly, and `new`
            //   set the ROM's flash descriptor to cover at least `self.size`
            //   bytes. `self.offset` checked that `offset` is below
            //   `self.size`, which is a multiple of `PAGE_SIZE`, so the page
            //   lies on the flash.
            //
            // - Written bytes: `offset` is a multiple of `PAGE_SIZE`, so the
            //   chunk is one whole, aligned 256-byte page of the flash. `new`'s
            //   caller guarantees that the driver's users only write pages that
            //   hold nothing the kernel or processes depend on.
            //
            // - Source buffer: `chunk` starts at a multiple of 256 bytes into
            //   `buf`, which `Esp32C3Page` aligns to 4 bytes, and `buf` stays
            //   borrowed for the call. `buf` is a `&'static mut`, so it is in
            //   writable memory, and not in the read-only XIP mapping of the
            //   flash.
            let status = unsafe {
                write(
                    (offset + i * PROGRAM_SIZE) as u32,
                    chunk.as_ptr().cast(),
                    PROGRAM_SIZE as u32,
                )
            };

            if status != 0 {
                result = Err(hil::flash::Error::FlashError);
                break;
            }
        }

        self.buffer.replace(buf);

        // This schedules a deferred call, as we can't synchronously invoke the
        // callback here:
        self.complete(Operation::Write, result);
        Ok(())
    }

    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode> {
        if self.pending.get().is_some() {
            return Err(ErrorCode::BUSY);
        }
        let Some(offset) = self.offset(page_number) else {
            return Err(ErrorCode::INVAL);
        };

        // SAFETY: This upholds the safety conditions of `erase_sector`:
        //
        // - ROM addresses, `.iram` contents, machine mode and the ROM's
        //   reserved SRAM: `new`'s caller guarantees these.
        //
        // - Flash mapping and descriptor: `new`'s caller guarantees that the
        //   mask ROM maps the flash linearly, and `new` set the ROM's flash
        //   descriptor to 4 KiB sectors covering at least `self.size` bytes.
        //   `self.offset` checked that `offset` is below `self.size`, so the
        //   sector lies on the flash.
        //
        // - Erased sector: each page is one sector. `new`'s caller guarantees
        //   that the driver's users only erase pages that hold nothing the
        //   kernel or processes depend on.
        let status = unsafe { erase_sector((offset / PAGE_SIZE) as u32) };

        let result = if status == 0 {
            Ok(())
        } else {
            Err(hil::flash::Error::FlashError)
        };

        // This schedules a deferred call, as we can't synchronously invoke the
        // callback here:
        self.complete(Operation::Erase, result);

        Ok(())
    }
}

impl DeferredCallClient for SpiFlash {
    fn handle_deferred_call(&self) {
        let Some((operation, result)) = self.pending.take() else {
            return;
        };

        self.client.map(|client| match operation {
            Operation::Read => {
                self.buffer
                    .take()
                    .map(|buf| client.read_complete(buf, result));
            }

            Operation::Write => {
                self.buffer
                    .take()
                    .map(|buf| client.write_complete(buf, result));
            }

            Operation::Erase => client.erase_complete(result),
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
