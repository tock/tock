/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

MEMORY
{
  /* uncomment this to boot from RAM */
  /* reset (rx)  : ORIGIN = 0x20000000, LENGTH = 16K
  rom (rx)  : ORIGIN = 0x20000100, LENGTH = 256K
  prog (rx) : ORIGIN = 0x20040000, LENGTH = 1K
  ram (rwx) : ORIGIN = 0x20004000, LENGTH = 240K */

  /* boot from Flash */
  rom (rx)  : ORIGIN = 0x10000000, LENGTH = 256K
  prog (rx) : ORIGIN = 0x10040000, LENGTH = 256K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 264K
}

PAGE_SIZE = 4K;

ENTRY(jump_to_bootloader)

/* RP2040 has a bootloader that occupies the first 256 bytes of .text */
SECTIONS {
  .text  : ALIGN(4)
    {
        KEEP (*(.flash_bootloader));
        . = ALIGN (256);
    } > rom 
}

INCLUDE ../kernel_layout.ld
