/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* Memory layout for the STM32F401CCU6
 * rom = 256KiB (LENGTH = 0x00040000)
 * kernel = 150KiB
 * user = 102KiB
 * ram = 64KiB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 150K
  prog (rx) : ORIGIN = 0x08025800, LENGTH = 102K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}

PAGE_SIZE = 2K;

INCLUDE ../kernel_layout.ld
