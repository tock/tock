/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2026.                                  */

/* Memory layout for NXP S32G3 Main SoC M7_0.
 *
 * Single link base: 0x34200000.  This address is within the M7_0 SRAM region
 * that is accessible after reset.
 *
 * Writable data could live in DTCM (zero-wait, ECC-protected) so the kernel
 * never depends on L2 SRAM staging beyond the initial code/rodata.
 * but this area is too small, and the tock default linker script has no option to separate kernel data into a different section.
 * So we place the kernel data in L2 SRAM, which is larger but slower.
 */

MEMORY
{
  rom  (rx)  : ORIGIN = 0x34200000, LENGTH = 0x00020000  /* 128 KiB — kernel code + rodata */
  prog (rx)  : ORIGIN = 0x34220000, LENGTH = 0x00080000  /* 512 KiB — TBF app area */
  ram  (rwx) : ORIGIN = 0x34000000, LENGTH = 0x00200000  /* 2 MiB - stack, BSS, data */
}
