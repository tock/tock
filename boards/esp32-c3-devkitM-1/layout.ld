/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* The ESP32-C3 supports multiple differnet boot-modes, implemented in the
 * integrated ROM bootloader. For Tock, we configure it as follows:
 *
 * The kernel and apps execute in place from flash. We use the mask ROM's
 * direct boot mode by starting our binary image with two magic words
 * (`0xaedb041d`, `0xaedb041d`). This causes the mask ROM to map the external
 * SPI flash chip at address 0x42000000 and then jumps to 0x42000008.
 * The mask ROM reserves the top of SRAM for its own internal data (starting
 * from 0x3FCDF000); we are free to use any SRAM up until that address.
 *
 * The `.iram` section holds code that must not execute from flash. Instead,
 * this code is copied from flash into SRAM during early boot. The ESP32-C3
 * implements a modified harvard architecture where SRAM addresses cannot be
 * executed directly, but the CPU can execute instructions in SRAM at an alias
 * mapped at `SRAM_BASE (0x3FC8_0000) + 0x700000 = IRAM_BASE (0x4000_0000)`.
 * Similar to the kernel's `.data` section, we re-locate it on boot through a
 * loader stub that is linked at `.esp_direct_boot.entry` and thus executed as
 * the first function boot. This then chains the remaining, shared RISC-V start
 * assembly.
 *
 * The kernel stack is at the bottom of the `ram` region. The `iram` region is
 * at the top of the usable SRAM because a kernel stack overflow must not
 * overwrite code in the `iram` region.
 */

MEMORY
{
  rom  (rx)  : ORIGIN = 0x42000000, LENGTH = 0x00100000
  prog (rx)  : ORIGIN = 0x42100000, LENGTH = 0x00300000
  ram  (rw)  : ORIGIN = 0x3FC80000, LENGTH = 0x0005EF00
  iram (rx)  : ORIGIN = 0x403DEF00, LENGTH = 0x00000100
}

SECTIONS
{
  .esp_direct_boot :
  {
    LONG(0xaedb041d)
    LONG(0xaedb041d)
    KEEP(*(.esp_direct_boot.entry))
  } > rom
}

INCLUDE tock_kernel_layout.ld

SECTIONS
{
  .iram :
  {
    . = ALIGN(4);
    *(.iram .iram.*)
    . = ALIGN(4);
  } > iram AT> rom
}
INSERT AFTER .text;

/* The instruction bus is read-only. The `esp32_c3_direct_boot_entry` chip crate
 * entry stub writes the `.iram` section through the data bus. */
_siram_load = LOADADDR(.iram);
_siram_dbus = ADDR(.iram) - 0x700000;
_eiram_dbus = ADDR(.iram) + SIZEOF(.iram) - 0x700000;
