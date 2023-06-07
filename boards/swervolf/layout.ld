/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 0x30000
  prog (rx) : ORIGIN = 0x30000,    LENGTH = 0x30000
  ram (rwx) : ORIGIN = 0x60000,    LENGTH = 0x30000
}

INCLUDE ../kernel_layout.ld
