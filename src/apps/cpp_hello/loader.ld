ENTRY(_start)

MEMORY {
    FLASH (rx) : ORIGIN = 0x80000000, LENGTH = 0x00080000
    SRAM (RWX) : ORIGIN = 0x00000000, LENGTH = 0x00002000
}

SECTIONS {
/* Load information, used by runtime to load app */
    .load_info :
    {
        KEEP(*(.load_info))
    } > FLASH =0xFF

/* Text section, Code! */
    .text :
    {
        _text = .;
        KEEP (*(.start))
        *(.text*)
        *(.rodata*)
        KEEP (*(.syscalls))
    } > FLASH =0xFF

    /* .ARM.exidx is sorted, so has to go in its own output section.  */
    PROVIDE_HIDDEN (__exidx_start = .);
    .ARM.exidx :
    {
      *(.ARM.exidx* .gnu.linkonce.armexidx.*)
      _etext = .;
    } > FLASH =0xFF
    PROVIDE_HIDDEN (__exidx_end = .);

/* Global Offset Table */
    .got :
    {
        _got = .;
        *(.got*)
        _egot = .;
        _plt = .;
        *(.got.plt*)
        _eplt = .;
    } > SRAM AT > FLASH

/* Data section, static initialized variables
 *  Note: This is placed in Flash after the text section, but needs to be
 *  moved to SRAM at runtime
 */
    .data :
    {
        _data = .;
        *(.data*)
        _edata = .;
    } > SRAM AT > FLASH

/* BSS section, static uninitialized variables */
    .bss :
    {
        _bss = .;
        *(.bss*)
        *(COMMON)
        _ebss = .;
    } > SRAM
}
