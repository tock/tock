/* Userland Generic Layout
 *
 * This linker script is designed for Tock apps where the end microcontroller
 * is not known. Therefore, this script over provisions space on some platforms.
 */

/* Memory Spaces Definitions, 448K flash, 64K ram */
PROG_LENGTH = 0x00040000;
RAM_LENGTH  = 0x00010000;

ENTRY(_start)

/* Note: Because apps are relocated, the FLASH address here acts as a sentinel
 * value for relocation fixup routines. The application loader will select the
 * actual location in flash where the app is placed.
 */
MEMORY {
    FLASH (rx) : ORIGIN = 0x80000000, LENGTH = PROG_LENGTH
    SRAM (RWX) : ORIGIN = 0x00000000, LENGTH = RAM_LENGTH
}

SECTIONS {
    /* Text section, Code! */
    .text :
    {
        _text = .;
        /**
         * Populate the header expected by `crt0`:
         *
         *
         *  struct hdr {
         *    uint32_t got_sym_start;
         *    uint32_t got_start;
         *    uint32_t got_size;
         *    uint32_t data_sym_start;
         *    uint32_t data_start;
         *    uint32_t data_size;
         *    uint32_t bss_start;
         *    uint32_t bss_size;
         *    uint32_t reldata_start;
         *    uint32_t stack_size;
         *  };
         */
        /* Offset of GOT symbols in flash */
        LONG(LOADADDR(.got) - _text);
        /* Offset of GOT section in memory */
        LONG(_got);
        /* Size of GOT section */
        LONG(SIZEOF(.got));
        /* Offset of data symbols in flash */
        LONG(LOADADDR(.data) - _text);
        /* Offset of data section in memory */
        LONG(_data);
        /* Size of data section */
        LONG(SIZEOF(.data));
        /* Offset of BSS section in memory */
        LONG(_bss);
        /* Size of BSS section */
        LONG(SIZEOF(.bss));
        /* First address offset after program flash, where elf2tbf places
         * .rel.data section */
        LONG(LOADADDR(.endsec) - _text);
        /* The size of the stack requested by this application */
        LONG(STACK_SIZE);

        KEEP (*(.start))
        *(.text*)
        *(.rodata*)
        KEEP (*(.syscalls))
        _etext = .;
        *(.ARM.extab*)
        . = ALIGN(4); /* Make sure we're word-aligned here */
    } > FLASH =0xFF


    /* App state section. Used for persistent app data. */
    .app_state :
    {
        KEEP (*(.app_state))
        . = ALIGN(4); /* Make sure we're word-aligned here */
    } > FLASH =0xFF

    /* Global Offset Table */
    .got :
    {
        _got = .;
        *(.got*)
        *(.got.plt*)
        . = ALIGN(4); /* Make sure we're word-aligned here */
    } > SRAM AT > FLASH

    /* Data section, static initialized variables
     *  Note: This is placed in Flash after the text section, but needs to be
     *  moved to SRAM at runtime
     */
    .data :
    {
        _data = .;
        KEEP(*(.data*))
        . = ALIGN(4); /* Make sure we're word-aligned here */
    } > SRAM AT > FLASH

    /* BSS section, static uninitialized variables */
    .bss :
    {
        _bss = .;
        KEEP(*(.bss*))
        *(COMMON)
        . = ALIGN(4); /* Make sure we're word-aligned here */
    } > SRAM

    /*
     * __NOTE__: The following symbols are used only to hint to elf2tbf how much
     * total memory to request from the OS.
     */
    .stack :
    {
        . += STACK_SIZE;
        . = ALIGN(4); /* Make sure we're word-aligned here */
    } > SRAM

    .app_heap :
    {
        . += APP_HEAP_SIZE;
        . = ALIGN(4); /* Make sure we're word-aligned here */
    } > SRAM

    .kernel_heap :
    {
        . += KERNEL_HEAP_SIZE;
        . = ALIGN(4); /* Make sure we're word-aligned here */
    } > SRAM

    _sram_end = .;
    .endsec :
    {
    } > FLASH

    /* ARM Exception support
     *
     * This contains compiler-generated support for unwinding the stack,
     * consisting of key-value pairs of function addresses and information on
     * how to unwind stack frames.
     * https://wiki.linaro.org/KenWerner/Sandbox/libunwind?action=AttachFile&do=get&target=libunwind-LDS.pdf
     *
     * .ARM.exidx is sorted, so has to go in its own output section.
     *
     * __NOTE__: It's at the end because we currently don't actually serialize
     * it to the binary in elf2tab. If it was before the RAM sections, it would
     * through off our calculations of the header.
     */
    PROVIDE_HIDDEN (__exidx_start = .);
    .ARM.exidx :
    {
      /* (C++) Index entries for section unwinding */
      *(.ARM.exidx* .gnu.linkonce.armexidx.*)
    } > FLASH
    PROVIDE_HIDDEN (__exidx_end = .);

}
