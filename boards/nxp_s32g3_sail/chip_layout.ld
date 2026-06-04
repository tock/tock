/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */

/* Memory layout for NXP S32G3 Main SoC M7_0 (XMODEM target).
 *
 * The XMODEM bootloader deposits the payload at 0x34000000 (SRAM).
 * LMA == VMA for all sections — no load-time copy is required.
 *
 * Stack and writable data live in DTCM (0x20000000, 56 KiB), which is
 * separate from the code SRAM and survives across soft resets.
 */

MEMORY
{
  rom  (rx)  : ORIGIN = 0x34000000, LENGTH = 0x0003F000  /* 252 KiB — kernel code + rodata */
  prog (rx)  : ORIGIN = 0x3403F000, LENGTH = 0x00008000  /* 32 KiB  — TBF app area          */
  ram  (rwx) : ORIGIN = 0x20000000, LENGTH = 0x0000E000  /* 56 KiB  — DTCM: stack, BSS, data */
}
