/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* Memory layout for the STM32F401CCU6
 * rom = 256KiB (LENGTH = 0x00040000)
 * kernel = 192KiB
 * user = 64KiB
 * ram = 64KiB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00030000
  prog (rx) : ORIGIN = 0x08030000, LENGTH = 0x00010000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}

PAGE_SIZE = 2K;

INCLUDE ../kernel_layout.ld
