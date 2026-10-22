/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2026.                                  */

/* The secure stage takes the first 16K of flash, one SPU region, which holds
 * the reset vector and ends with the non-secure callable gateway. Its stack
 * is in the first 8K of RAM, one SPU region, which stays secure. */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 16K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 8K
}

ENTRY(reset_handler)

SECTIONS
{
  .text :
  {
    /* Vector table: the initial stack pointer, the reset handler, then
     * NMI through SysTick. */
    LONG(ORIGIN(ram) + LENGTH(ram))
    LONG(reset_handler)
    LONG(fault_handler) LONG(fault_handler) LONG(fault_handler)
    LONG(fault_handler) LONG(fault_handler) LONG(fault_handler)
    LONG(fault_handler) LONG(fault_handler) LONG(fault_handler)
    LONG(fault_handler) LONG(fault_handler) LONG(fault_handler)
    LONG(fault_handler) LONG(fault_handler)
    *(.text .text.*)
    *(.rodata .rodata.*)
  } > rom

  /* The gateway's entry points, in the window the SPU makes non-secure
   * callable. */
  .nsc ORIGIN(rom) + LENGTH(rom) - 32 : { KEEP(*(.nsc)) } > rom

  .data : { *(.data .data.*) } > ram AT > rom
  /* Only statics of the kernel code below, which never runs, land here. */
  .bss (NOLOAD) : { *(.bss .bss.* COMMON) } > ram

  /* Also drop the kernel's vector table and attributes, which the chip and
   * kernel crates provide. */
  /DISCARD/ : { *(.ARM.exidx .ARM.exidx.* .ARM.extab.* .vectors .irqs .tock.attr.*) }
}

/* The kernel's startup and exception code in the arch crate is linked in, as
 * exported functions, but never runs. Give the symbols it refers to values so
 * the link succeeds. */
PROVIDE(main = fault_handler);
PROVIDE(_estack = ORIGIN(ram) + LENGTH(ram));
PROVIDE(_etext = 0);
PROVIDE(_srelocate = 0);
PROVIDE(_erelocate = 0);
PROVIDE(_szero = 0);
PROVIDE(_ezero = 0);

ASSERT(SIZEOF(.nsc) == 32, "The gateway has four 8-byte entry points")
ASSERT(SIZEOF(.data) == 0,
       "The secure stage does not initialize RAM, so it must not use .data")
