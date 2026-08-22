/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2026.                                  */

/*
 * SHAKTI C-Class test SoC: 2 GiB of RAM starting at 0x80000000. The
 * boot ROM jumps to 0x80000000, so the kernel is placed at the start of DRAM
 * (same convention as the the bare-metal bring-up program).
 */

MEMORY
{
  rom  (rx)  : ORIGIN = 0x80000000, LENGTH = 0x00100000
  prog (rx)  : ORIGIN = 0x80100000, LENGTH = 0x00100000
  ram  (rwx) : ORIGIN = 0x80200000, LENGTH = 0x01000000
}

INCLUDE tock_kernel_layout.ld
