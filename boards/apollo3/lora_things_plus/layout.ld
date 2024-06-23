/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* We have to reduce all flash lengths by 0x2000 as that is the offset
 * we use when writing data to flash with ambiq_bin2board.py.
 */

MEMORY
{
  rom (rx)  : ORIGIN = 0x0000C000, LENGTH = 0x034000 - 0x2000
  prog (rx) : ORIGIN = 0x00040000, LENGTH = 0x100000 - 0x040000 - 0x2000
  ram (rwx) : ORIGIN = 0x10000000, LENGTH = 0x60000
}

PAGE_SIZE = 8K;

INCLUDE ../../kernel_layout.ld
