/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2025.                                  */

/* Board-specific linker script for nRF52840DK secure bootloader */

MEMORY
{
  /* Bootloader occupies first 32KB of flash */
  FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 32K
  RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 16K
}

/* Include common bootloader linker script */
INCLUDE ../../bootloader_layout.ld