/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* Memory layout for the STM32F412G
 * rom = 1MB (LENGTH = 0x01000000)
 * kernel = 256KB
 * user = 744KB
 * ram = 256KB */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00040000
  prog (rx) : ORIGIN = 0x08040000, LENGTH = 0x000C0000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x0003FFFF
}
