/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

MEMORY
{
  # with bootloader
  rom (rx)  : ORIGIN = 0x00008000, LENGTH = 224K
  # without bootloader
  # rom (rx)  : ORIGIN = 0x00000000, LENGTH = 256K
  prog (rx) : ORIGIN = 0x00040000, LENGTH = 256K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 128K
}

PAGE_SIZE = 4K;

INCLUDE tock_kernel_layout.ld
