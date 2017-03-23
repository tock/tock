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
