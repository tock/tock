/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2026.                                  */

/* Memory layout for NXP S32G3 Main SoC M7_0.
 *
 * Single link base: 0x34200000.  This address is within the M7_0 SRAM region
 * that is accessible after reset.
 *
 * DTCM is deliberately unused: its 56 KiB capacity cannot hold the kernel's
 * writable image, and splitting those sections would unnecessarily specialize
 * Tock's default linker layout. L2 SRAM holds the stack, BSS, and data.
 */

MEMORY
{
  rom  (rx)  : ORIGIN = 0x34200000, LENGTH = 0x00020000  /* 128 KiB — kernel code + rodata */
  prog (rx)  : ORIGIN = 0x34220000, LENGTH = 0x00080000  /* 512 KiB — TBF app area */
  ram  (rwx) : ORIGIN = 0x34000000, LENGTH = 0x00200000  /* 2 MiB - stack, BSS, data */
}
