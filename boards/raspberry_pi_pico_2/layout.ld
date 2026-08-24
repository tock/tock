/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright OxidOS Automotive 2025.                                  */

MEMORY
{
  /* To boot from RAM, uncomment this block and comment out the flash block
     below; the two cannot both be defined. The regions partition the 520K of
     SRAM, and ram is the largest because it holds the kernel stack, kernel
     data and all process RAM.

     These addresses are necessary but not sufficient. The RP2350 bootrom takes
     the vector table address from the IMAGE_DEF metadata block, a byte array
     in src/flash_bootloader.rs that cannot see this file, so it has to be
     relocated by hand to match rom below:

       METADATA_BLOCK bytes 12..16, little endian
         0x00, 0x04, 0x00, 0x10  =  0x10000400  boot from flash
         0x00, 0x04, 0x00, 0x20  =  0x20000400  boot from RAM

     Without that change the image loads into SRAM correctly and the bootrom
     then jumps into flash, which hangs with no output at all. */
  /* bootloader (rx) : ORIGIN = 0x20000000, LENGTH = 1024
  rom (rx)  : ORIGIN = 0x20000400, LENGTH = 127K
  prog (rx) : ORIGIN = 0x20020000, LENGTH = 8K
  ram (rwx) : ORIGIN = 0x20022000, LENGTH = 384K */

  /* boot from Flash */
  bootloader (rx) : ORIGIN = 0x10000000, LENGTH = 1024
  rom (rx)  : ORIGIN = 0x10000400, LENGTH = 255K
  prog (rx) : ORIGIN = 0x10040000, LENGTH = 256K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 520K
}

PAGE_SIZE = 4K;

ENTRY(jump_to_bootloader)

SECTIONS {
  .bootloader  : ALIGN(4)
    {
        KEEP (*(.metadata_block));
        KEEP (*(.flash_bootloader));
        . = ALIGN (256);
    } > bootloader
}

INCLUDE tock_kernel_layout.ld
