/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2024.                                  */
/* Copyright OxidOS Automotive 2026.                                  */

/* Memory layout for the STM32U545 (Secure Mode Aliases)
 * FLASH: 512 KB Total
 * KERNEL: 256 KB
 * APPS: 256 KB
 * RAM: 160 KB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x0C000000, LENGTH = 0x00040000 /* 256 KB Kernel */
  prog (rx) : ORIGIN = 0x0C040000, LENGTH = 0x00040000 /* 256 KB Apps */
  ram (rwx) : ORIGIN = 0x30000000, LENGTH = 0x00028000 /* 160 KB */
}

/* 
 * Tock's generic linker script (tock_kernel_layout.ld) usually defines
 * _estack at the START of RAM for overflow protection.
 *
 * However, for this board in Secure Mode, we need it at the END of RAM
 * to prevent an immediate HardFault.
 */
_estack = ORIGIN(ram) + LENGTH(ram);

PAGE_SIZE = 8K;
