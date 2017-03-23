ENTRY(_start)

/* Userland Layout
 *
 * This script is designed to be used in conjunction with a chip_layout.ld from
 * a Tock board directory. A properly configured layout.ld file will look like:
 *
 *     INCLUDE <tock_board_dir>/chip_layout.ld
 *     INCLUDE <tock_userland_base_dir>./userland_layout.ld
 *
 * The Tock build system should generate and use an appropriate linker script
 * automatically for you.
 */



/* TODO: Reserve space for Tock.
 *
 * Currently, this memory map allows applications to grow to the entire SRAM
 * size of the chip they are being built for. If an application is actually
 * that large, it will fail on loading because the Tock kernel requires some
 * amount of memory itself.
 *
 * I *believe* that we could lookup the location of the `.app_memory` section
 * from the built tock kernel image to get a better size estimate, but that
 * requires the userland build to know the location of the Tock kernel image
 * which isn't necessarily trivial.
 *
 * For now, the best we can do is fail at app load time if there's not enough
 * memory left for the app.
 *
 * TOCK_RAM_RESERVED = 0x4000;
 * SRAM (RWX) : ORIGIN = 0x00000000, LENGTH = RAM_LENGTH - TOCK_RAM_RESERVED
 *
 */


/* Note: Because apps are relocated, the FLASH address here acts as a sentinel
 * value for relocation fixup routines. The application loader will select the
 * actual location in flash where the app is placed.
 */
MEMORY {
    FLASH (rx) : ORIGIN = 0x80000000, LENGTH = PROG_LENGTH
    SRAM (RWX) : ORIGIN = 0x00000000, LENGTH = RAM_LENGTH
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

/* Beginning of SRAM */
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

/*
 * __NOTE__: The following symbols are used only to pass information
 * through the elf -> tbf -> Tock kernel.
 *
 * The kernel will place the stack at the beginning of the SRAM section so
 * that stack overflows run off the end of the memory segment and trigger an
 * MPU violation instead of overwriting data/got/bss information. This means
 * the actual location of symbols in those sections in memory will be offset
 * by STACK_SIZE.
 */
    .stack :
    {
        _stack = .;
        . += STACK_SIZE;
        _estack = .;
    } > SRAM

    .app_heap :
    {
        _app_heap = .;
        . += APP_HEAP_SIZE;
        _eapp_heap = .;
    } > SRAM

    .kernel_heap :
    {
        _kernel_heap = .;
        . += KERNEL_HEAP_SIZE;
        _ekernel_heap = .;
    } > SRAM

    _sram_end = .;
}
