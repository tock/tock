MEMORY {
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 192K
  prog (rx) : ORIGIN = 0x00030000, LENGTH = 320K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}
__stack_size__ = DEFINED(__stack_size__) ? __stack_size__ : 0x1000;
SECTIONS {
  .text : {
    KEEP(*(.vectors))
    KEEP(*(.irqs))
    *(.text .text.*)
    *(.rodata*)
    *(.ARM.extab* .gnu.linkonce.armextab.*)
  } > rom

  .apps :
  {
    _sapps = .;
    KEEP(*(.app.*))
    LONG(0) /* Marks the end of the app list with a sentinel value. */
  } > prog

  /* .ARM.exidx is sorted, so has to go in its own output section.  */
  PROVIDE_HIDDEN (__exidx_start = .);
  .ARM.exidx :
  {
    *(.ARM.exidx* .gnu.linkonce.armexidx.*)
  } > rom
  PROVIDE_HIDDEN (__exidx_end = .);

  _etext = .;

  .data : AT(_etext) {
    . = ALIGN(4);
    _srelocate = .;
    *(.data .data.*)
    . = ALIGN(4);
    _erelocate = .;
  } > ram

  .bss (NOLOAD) : {
    . = ALIGN(4);
    _szero = .;
    *(.bss .bss.*)
    _ezero = .;

    _sstack = .;
    . = . + __stack_size__;
    . = ALIGN(8);
    _estack = .;

    . = ALIGN(8K);
    *(.app_memory)
  } > ram

}
