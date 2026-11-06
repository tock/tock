/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2026.                                  */

/* nRF5340 network core, 256K flash, 64K ram */
MEMORY
{
  rom (rx)  : ORIGIN = 0x01000000, LENGTH = 128K
  prog (rx) : ORIGIN = 0x01020000, LENGTH = 128K
  ram (rwx) : ORIGIN = 0x21000000, LENGTH = 64K
}

PAGE_SIZE = 2K;

INCLUDE tock_kernel_layout.ld
