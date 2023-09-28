/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* Memory Spaces Definitions, 448K flash, 64K ram */
/* Bootloader is at address 0x00000000 */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00010000, LENGTH = 0x00020000
  prog (rx) : ORIGIN = 0x00030000, LENGTH = 0x00040000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}
