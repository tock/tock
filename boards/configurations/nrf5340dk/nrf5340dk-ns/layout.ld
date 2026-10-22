/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2026.                                  */

/* nRF5340 application core in non-secure state. The secure stage takes the
 * first 16K of flash, one SPU region. The last 8K of RAM (0x2007E000) is the
 * BLE mailbox shared with the network core. */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00004000, LENGTH = 240K
  prog (rx) : ORIGIN = 0x00040000, LENGTH = 768K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 504K
}

PAGE_SIZE = 4K;

INCLUDE tock_kernel_layout.ld
