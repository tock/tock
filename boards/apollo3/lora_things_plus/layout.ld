/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* We have to reduce all flash lengths by 0x2000 as that is the offset
 * we use when writing data to flash with ambiq_bin2board.py.
 */

MEMORY
{
  rom (rx)     : ORIGIN = 0x0000C000, LENGTH = 0x034000 - 0x2000
  /* We reserve 0x28000 bytes at the end for K/V data */
  prog (rx)    : ORIGIN = 0x00040000, LENGTH = 0xC0000 - 0x2000 - 0x28000
  kv_data (rx) : ORIGIN = 0x000D6000, LENGTH = 0x28000
  ram (rwx)    : ORIGIN = 0x10000000, LENGTH = 0x60000
}

_skv_data = ORIGIN(kv_data);
_lkv_data = LENGTH(kv_data);

PAGE_SIZE = 8K;

INCLUDE tock_kernel_layout.ld
