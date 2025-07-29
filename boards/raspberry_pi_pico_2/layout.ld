/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright OxidOS Automotive 2025.                                  */

MEMORY
{
  /* uncomment this to boot from RAM */
  /* reset (rx)  : ORIGIN = 0x20000000, LENGTH = 16K
  rom (rx)  : ORIGIN = 0x20000100, LENGTH = 256K
  prog (rx) : ORIGIN = 0x20040000, LENGTH = 1K
  ram (rwx) : ORIGIN = 0x20004000, LENGTH = 240K */

  /* boot from Flash */
  bootloader (rx) : ORIGIN = 0x10000000, LENGTH = 1024
  rom (rx)  : ORIGIN = 0x10000400, LENGTH = 255K
  prog (rx) : ORIGIN = 0x10040000, LENGTH = 256K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 264K
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

    /*
     * This padding has been added because arm-none-eabi-objcopy
     * wrongly aligns the .stack section. This is a workaround, not a
     * permanent fix. See https://github.com/tock/tock/pull/4416#discussion_r2237885703
     * for details.
     */
   .padding (NOLOAD) : {
        BYTE(0xFF)
    } > ram
}

INCLUDE tock_kernel_layout.ld
