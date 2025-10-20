/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2025.                                  */

MEMORY
{
  rom(rx) : ORIGIN = 0x00000000, LENGTH = 128K
  ram(rwx) : ORIGIN = 0x20000000, LENGTH = 320K
  prog(rx) : ORIGIN = 0x00020000, LENGTH = 512K
}

PAGE_SIZE = 4K;


INCLUDE tock_kernel_layout.ld
