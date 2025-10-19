/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2025.                                  */

/* Memory layout for the STM32L476RG
 * rom = 512KB (LENGTH = 0x00080000)
 * kernel = 256KB
 * user = 256KB
 * ram = 128KB */

/* Memories definition */
MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00040000
  prog (rx) : ORIGIN = 0x08040000, LENGTH = 0x00040000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00020000
}

PAGE_SIZE = 2K;
