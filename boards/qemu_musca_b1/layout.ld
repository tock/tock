/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */


MEMORY
{
  rom (rx)  : ORIGIN = 0x10000000, LENGTH = 1M
  
  prog (rx) : ORIGIN = 0x10100000, LENGTH = 1M
  
  ram (rwx) : ORIGIN = 0x30000000, LENGTH = 512K
}

/* PAGE_SIZE = 2K; */

INCLUDE tock_kernel_layout.ld
