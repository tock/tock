/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/**
 * QEMU emulated DRAM region. Tock is currently designed to be placed
 * at the start of DRAM, using the `-bios` option in qemu-system-riscv32.
 *
 * We are using 4MB of RAM, which easily fits into the 128MB default
 * assignment of QEMU, and we can have compact VMs with `-m 4MB`
 */

MEMORY
{
  rom (rx)  : ORIGIN = 0x80000000, LENGTH = 0x100000
  prog (rx) : ORIGIN = 0x80100000, LENGTH = 0x100000
  ram (rwx) : ORIGIN = 0x80200000, LENGTH = 0x200000
}

SECTIONS {
    /* Export the start & end of SRAM and flash as symbols for setting
     * up the ePMP. Flash includes rom, prog and flash storage, such
     * that we can use a single NAPOT region. The .text section will
     * be made executable by a separate PMP region.
     */
    _sflash = ORIGIN(rom);
    _eflash = ORIGIN(prog) + LENGTH(prog);

    _ssram  = ORIGIN(ram);
    _esram  = ORIGIN(ram) + LENGTH(ram);
}

INCLUDE tock_kernel_layout.ld
