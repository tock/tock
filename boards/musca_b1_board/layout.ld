/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */


MEMORY
{
  rom (rx)  : ORIGIN = 0x10000000, LENGTH = 0x100000
  
  prog (rx) : ORIGIN = 0x10100000, LENGTH = 0x100000
  
  ram (rwx) : ORIGIN = 0x30000000, LENGTH = 0x30000
}

/* PAGE_SIZE = 2K; */

INCLUDE tock_kernel_layout.ld
