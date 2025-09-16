/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2025.                                  */

MEMORY
{
  rom(rx) : ORIGIN = 0x00000000, LENGTH = 64K
  ram(rwx) : ORIGIN = 0x20000000, LENGTH = 320K
  prog(rx) : ORIGIN = 0x00010000, LENGTH = 576K
}

PAGE_SIZE = 4K;

SECTIONS{
  .stack (NOLOAD) :
{
    . = ALIGN(8);
    *(.stack*)
    . = ALIGN(8);
} > ram

}

ENTRY(initialize_ram_jump_to_main)
