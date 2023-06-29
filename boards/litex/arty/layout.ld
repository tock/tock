/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* The entire memory is actually placed into DRAM by the bootloader */

MEMORY
{
  rom (rx)  : ORIGIN = 0x40000000, LENGTH = 0x1000000
  prog (rx) : ORIGIN = 0x41000000, LENGTH = 0x1000000
  ram (rwx) : ORIGIN = 0x42000000, LENGTH = 0x50000000 - 0x42000000
}

INCLUDE ../../kernel_layout.ld
