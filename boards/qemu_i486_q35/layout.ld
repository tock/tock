/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2024.                                  */

/**
 * QEMU emulated DRAM region. This board is designed to be loaded as a Multiboot V1-compliant ELF
 * file, which means we get to choose the desired memory map (within the bounds of DRAM provided by
 * QEMU).
 */

MEMORY
{
    rom   (rx)   : ORIGIN = 0x00100000, LENGTH = 0x50000
    prog  (rx)   : ORIGIN = 0x00150000, LENGTH = 0xC0000
    ram  (rwx)   : ORIGIN = 0x00210000, LENGTH = 0x40000
    pages (rw)   : ORIGIN = 0x00250000, LENGTH = 0x2000

	virtual_prog : ORIGIN = 0x00300000, LENGTH = 0x80000
	virtual_ram  : ORIGIN = 0x00380000, LENGTH = 0x80000
}

/**
 * Tock's x86 port uses MMU page tables to implement memory protection, so we must take care to have
 * memory regions aligned and padded to 4k boundaries.
 */
PAGE_SIZE = 4K;

SECTIONS
{
	_srom = ORIGIN(rom);
	_erom = ORIGIN(rom) + LENGTH(rom);

	_sprog = ORIGIN(prog);
	_eprog = ORIGIN(prog) + LENGTH(prog);

	_sram = ORIGIN(ram);
	_eram = ORIGIN(ram) + LENGTH(ram);

	_speripheral = ORIGIN(pages);
	_eperipheral = ORIGIN(pages) + LENGTH(pages);

    /**
     * Dedicated and aligned storage used for x86 page tables and directories.
     */
    .pages :
    {
        . = ALIGN(4096);
        *(.pde);
        . = ALIGN(4096);
        *(.pte);
    } > pages
}

INCLUDE tock_kernel_layout.ld
