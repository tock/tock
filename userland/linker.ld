ENTRY(_start)

MEMORY {
    FLASH (rx) : ORIGIN = 0x80000000, LENGTH = 0x00080000
    SRAM (RWX) : ORIGIN = 0x00000000, LENGTH = 0x00004000
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
        _etext = .;
    } > FLASH =0xFF

    _sram_start = .;

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
        KEEP(*(.data*))
        _edata = .;
    } > SRAM AT > FLASH

/* BSS section, static uninitialized variables */
    .bss :
    {
        _bss = .;
        KEEP(*(.bss*))
        *(COMMON)
        _ebss = .;
    } > SRAM

    .stack :
    {
        _stack = .;
        KEEP(*(.stack*))
        _estack = .;
    } > SRAM

    .app_heap :
    {
        _app_heap = .;
        KEEP(*(.app_heap*))
        _eapp_heap = .;
    } > SRAM

    .kernel_heap :
    {
        _kernel_heap = .;
        KEEP(*(.kernel_heap*))
        _ekernel_heap = .;
    } > SRAM

    _sram_end = .;
}
