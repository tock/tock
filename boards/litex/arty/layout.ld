/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* The entire memory is actually placed into DRAM by the bootloader */

MEMORY
{
  rom (rx)  : ORIGIN = 0x40000000, LENGTH = 0x1000000
  prog (rx) : ORIGIN = 0x41000000, LENGTH = 0x1000000
  ram (rwx) : ORIGIN = 0x42000000, LENGTH = 0x1000000
}

SECTIONS {
    /* Export the start & end of SRAM and flash as symbols for setting up the
     * PMP. Flash includes rom and prog, such that we can use a single NAPOT
     * region. The .text section will be made executable by a separate PMP
     * region.
     */
    _sflash = ORIGIN(rom);
    _eflash = ORIGIN(prog) + LENGTH(prog);

    _ssram  = ORIGIN(ram);
    _esram  = ORIGIN(ram) + LENGTH(ram);
}

INCLUDE tock_kernel_layout.ld
