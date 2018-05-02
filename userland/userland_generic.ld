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
    /* Section for just the app crt0 header.
     * This must be first so that the app can find it.
     */
    .crt0_header :
    {
        _beginning = .; /* Start of the app in flash. */
        /**
         * Populate the header expected by `crt0`:
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
        LONG(LOADADDR(.got) - _beginning);
        /* Offset of GOT section in memory */
        LONG(_got);
        /* Size of GOT section */
        LONG(SIZEOF(.got));
        /* Offset of data symbols in flash */
        LONG(LOADADDR(.data) - _beginning);
        /* Offset of data section in memory */
        LONG(_data);
        /* Size of data section */
        LONG(SIZEOF(.data));
        /* Offset of BSS section in memory */
        LONG(_bss);
        /* Size of BSS section */
        LONG(SIZEOF(.bss));
        /* First address offset after program flash, where elf2tab places
         * .rel.data section */
        LONG(LOADADDR(.endflash) - _beginning);
        /* The size of the stack requested by this application */
        LONG(STACK_SIZE);
    } > FLASH =0xFF

    /* App state section. Used for persistent app data.
     * We put this first so that if the app code changes but the persistent
     * data doesn't, the app_state can be preserved.
     */
    .wfr.app_state :
    {
        KEEP (*(.app_state))
        . = ALIGN(4); /* Make sure we're word-aligned here */
    } > FLASH =0xFF

    /* Text section, Code! */
    .text :
    {
        . = ALIGN(4);
        _text = .;
        KEEP (*(.start))
        *(.text*)
        *(.rodata*)
        KEEP (*(.syscalls))
        _etext = .;
        *(.ARM.extab*)
        . = ALIGN(4); /* Make sure we're word-aligned here */
    } > FLASH =0xFF

    /* Global Offset Table */
    .got :
    {
        . = ALIGN(4); /* Make sure we're word-aligned here */
        _got = .;
        *(.got*)
        *(.got.plt*)
        . = ALIGN(4);
    } > SRAM AT > FLASH

    /* Data section, static initialized variables
     *  Note: This is placed in Flash after the text section, but needs to be
     *  moved to SRAM at runtime
     */
    .data :
    {
        . = ALIGN(4); /* Make sure we're word-aligned here */
        _data = .;
        KEEP(*(.data*))
        . = ALIGN(4); /* Make sure we're word-aligned at the end of flash */
    } > SRAM AT > FLASH

    /* BSS section, static uninitialized variables */
    .bss :
    {
        . = ALIGN(4); /* Make sure we're word-aligned here */
        _bss = .;
        KEEP(*(.bss*))
        *(COMMON)
        . = ALIGN(4);
    } > SRAM

    /* End of flash. */
    .endflash :
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
     * it to the binary in elf2tbf. If it was before the RAM sections, it would
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
