/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Copyright (c) 2024 Antmicro <www.antmicro.com>           */

MEMORY
{
  rom (rx)  : ORIGIN = 0x80000000, LENGTH = 0x30000
  prog (rx) : ORIGIN = 0x80030000, LENGTH = 0x30000
  ram (rwx) : ORIGIN = 0x80060000, LENGTH = 0x30000
}

INCLUDE ../kernel_layout.ld
