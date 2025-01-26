/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* RAM starts at 0x40000000, the binary is loaded into ROM at 0x0 */

MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 0x80000
  prog (rx) : ORIGIN = 0x00080000, LENGTH = 0x80000
  ram (rwx) : ORIGIN = 0x40000000, LENGTH = 0x10000000
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
