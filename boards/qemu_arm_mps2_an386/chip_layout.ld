/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2026.                                  */

/* Memory layout for the QEMU ARM MPS2 AN385 (Cortex-M3) machine.
 *
 * QEMU maps code/flash (SSRAM1) at 0x0, up to a hard 4 MiB cap
 * (`armv7m_load_kernel(..., 0, 0x400000)` in `hw/arm/mps2.c`), and RAM at
 * 0x21000000, backed by a fixed 16 MiB (`default_ram_size` in the same
 * file; QEMU errors if `-m` is overridden). We only claim a modest slice
 * of each; QEMU backs whatever the linker script declares regardless of
 * the machine's full capacity.
 *
 * rom = 256KB (kernel)
 * prog = 256KB (apps)
 * ram = 128KB
 */

MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 0x00040000
  prog (rx) : ORIGIN = 0x00040000, LENGTH = 0x00040000
  ram (rwx) : ORIGIN = 0x21000000, LENGTH = 0x00020000
}

PAGE_SIZE = 2K;
