/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

MEMORY
{
  rom (rx)  : ORIGIN = 0x40380000, LENGTH = 0x30000
  ram (rwx) : ORIGIN = 0x3FCA0000, LENGTH = 0x30000
  prog (rx) : ORIGIN = 0x403B0000, LENGTH = 0x30000
}

INCLUDE ../kernel_layout.ld
