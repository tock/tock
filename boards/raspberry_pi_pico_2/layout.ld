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
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 400K
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

SECTIONS {
    /* Core 1 kernel stack. Same size as core 0's (kernel::stack_size! in
     * main.rs); grows down from _estack_h1. */
    _sstack_h1 = ORIGIN(ram) + LENGTH(ram);
    _estack_h1 = _sstack_h1 + 0x3000;

    /* Core 1 app memory: the rest of the physically-present SRAM banks. */
    _sappmem_h1 = _estack_h1;
    _eappmem_h1 = 0x20000000 + 512K;
}

ASSERT(_eappmem_h1 > _sappmem_h1, "Stage A2: core 1 has no room left for app memory.")
