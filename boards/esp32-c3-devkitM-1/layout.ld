/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* The ESP32-C3 has some odd aliasing features in its memory, where
 * what we think is in flash is actually executing out of the same
 * address space as physical RAM.
 *
 * To convert from a "flash" address to the address it actually uses,
 * subtract 0x700000. The addresses in comments on the right indicate
 * the actual physical addresses used at runtime. These must not
 * overlap.
 */
MEMORY
{
  rom  (rx)  : ORIGIN = 0x40380000, LENGTH = 0x20000 /* 0x3FC80000 - 0x3FCA0000 */
  ram  (rwx) : ORIGIN = 0x3FCA0000, LENGTH = 0x18000 /* 0x3FCA0000 - 0x3FCB8000 */
  prog (rx)  : ORIGIN = 0x403B8000, LENGTH = 0x28000 /* 0x3FCB8000 - 0x3FCE0000 */
}

INCLUDE tock_kernel_layout.ld
