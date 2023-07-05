/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* Memory layout for the STM32F401CCU6
 * rom = 256KB (LENGTH = 0x00040000)
 * kernel = 128KB
 * user = 128KB
 * ram = 64KB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00020000
  prog (rx) : ORIGIN = 0x08020000, LENGTH = 0x00020000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}

PAGE_SIZE = 2K;

INCLUDE ../kernel_layout.ld
