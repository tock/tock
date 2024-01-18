/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

MEMORY
{
  rom (rx)  : ORIGIN = 0x0000C000, LENGTH = 0x00030000
  prog (rx) : ORIGIN = 0x00040000, LENGTH = 0x08000000 - 0x00030000
  ram (rwx) : ORIGIN = 0x10000000, LENGTH = 0x60000
}

PAGE_SIZE = 8K;

INCLUDE ../../kernel_layout.ld
