/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2026.                                  */

/* Memory layout skeleton for STM32U545.
 *
 * TODO: Replace with exact memory map for your STM32U545 variant and chosen
 * app partitioning.
 *
 * Example split below assumes:
 * - 2MB flash total
 * - 256KB kernel + 256KB app flash region
 * - 768KB SRAM total (placeholder)
 */

MEMORY
{
  rom (rx)  : ORIGIN = 0x08000000, LENGTH = 0x00040000
  prog (rx) : ORIGIN = 0x08040000, LENGTH = 0x00040000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x000C0000
}

PAGE_SIZE = 2K;
