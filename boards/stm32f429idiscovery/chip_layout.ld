/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* Memory layout for the STM32F446RE
 * rom = 2MB (LENGTH = 0x02000000)
 * kernel = 256KB
 * user = 256KB
 * ram = 192KB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00040000
  prog (rx) : ORIGIN = 0x08040000, LENGTH = 0x00040000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00030000
}

PAGE_SIZE = 2K;
