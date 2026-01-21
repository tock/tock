/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2025.                                  */

/* Common linker script for secure bootloader */

/* Entry point is the reset handler */
ENTRY(Reset_Handler)

SECTIONS
{
  /* Vector table must be at the start of flash */
  .vector_table ORIGIN(FLASH) :
  {
    KEEP(*(.vector_table))
  } > FLASH

  /* Code section */
  .text :
  {
    *(.text .text.*)
    *(.rodata .rodata.*)
  } > FLASH

  /* ARM exception unwinding (for panic handling) */
  .ARM.exidx :
  {
    *(.ARM.exidx* .gnu.linkonce.armexidx.*)
  } > FLASH

  /* Data section (initialized variables) */
  .data : AT(ADDR(.text) + SIZEOF(.text))
  {
    _sdata = .;
    *(.data .data.*)
    _edata = .;
  } > RAM

  /* Store the load address of .data in flash */
  _sidata = LOADADDR(.data);

  /* BSS section (uninitialized variables) - zeroed at boot */
  .bss :
  {
    _sbss = .;
    *(.bss .bss.*)
    *(COMMON)
    _ebss = .;
  } > RAM

  /* Stack grows downward from end of RAM */
  _stack_start = ORIGIN(RAM) + LENGTH(RAM);

  /* Discard unused sections to save space */
  /DISCARD/ :
  {
    *(.ARM.exidx)
    *(.ARM.exidx.*)
    *(.ARM.extab.*)
  }
}