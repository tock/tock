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
        _etext = .;
    } > FLASH =0xFF

/* ARM Exception support
 *
 * This contains compiler-generated support for unwinding the stack,
 * consisting of key-value pairs of function addresses and information on
 * how to unwind stack frames.
 * https://wiki.linaro.org/KenWerner/Sandbox/libunwind?action=AttachFile&do=get&target=libunwind-LDS.pdf
 *
 * .ARM.exidx is sorted, so has to go in its own output section.
 */
    PROVIDE_HIDDEN (__exidx_start = .);
    .ARM.exidx :
    {
      /* (C++) Index entries for section unwinding */
      *(.ARM.exidx* .gnu.linkonce.armexidx.*)
    } > FLASH
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
}
