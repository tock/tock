/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Copyright (c) 2024 Antmicro <www.antmicro.com>           */

MEMORY
{
  rom (rx)  : ORIGIN = 0x20000000, LENGTH = 0x300000
  prog (rx) : ORIGIN = 0x20300000, LENGTH = 0x300000
  ram (rwx) : ORIGIN = 0x20600000, LENGTH = 0x300000
}

INCLUDE tock_kernel_layout.ld
