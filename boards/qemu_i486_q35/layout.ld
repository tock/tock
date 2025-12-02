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
    rom   (rx) : ORIGIN = 0x00100000, LENGTH = 0x50000
    prog  (rx) : ORIGIN = 0x00150000, LENGTH = 0xC0000
    ram  (rwx) : ORIGIN = 0x00210000, LENGTH = 0x40000
    pages (rw) : ORIGIN = 0x00250000, LENGTH = 0x2000
}

/**
 * Tock's x86 port uses MMU page tables to implement memory protection, so we must take care to have
 * memory regions aligned and padded to 4k boundaries.
 */
PAGE_SIZE = 4K;

/**
 * The order matters here as we need to make sure that qemu finds the multiboot header within the
 * first 8K of the ELF. The multiboot header is placed at the beginning of the .text section, so
 * this section has to be is placed first in the file.
 *  
 * The multiboot header's documentation is available at
 * https://www.gnu.org/software/grub/manual/multiboot/multiboot.html.
 * This allows Tock to be booted using a bootloader that supports multiboot v1.
 * QEMU provides for x86 such a bootloader when used with the `-kernel` flag.
 *
 * The bootloader is responsable for loading the kernel into RAM and executing it. Without it,
 * the Tock kernel should provide a bootloader that starts reading the kernel into RAM.
 *
 * This is the reason why this INCLUDE has to be before the .pages declaration
 */

INCLUDE tock_kernel_layout.ld

SECTIONS
{
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

