/*
 * This is the generic linker script for Tock. For most developers, it should
 * be sufficient to define {ROM/PROG/RAM}_{ORIGIN/LENGTH} (6 variables, the
 * start and length for each) and MPU_MIN_ALIGN (the minimum alignment
 * granularity supported by the MPU).
 *
 * --------------------------------------------------------------------------
 *
 * If you wish to create your own linker script from scratch, you must define
 * the following symbols:
 *
 * `_etext`, `_srelocate`, `_erelocate`
 *    The `_etext` symbol marks the end of data stored in flash that should
 *    stay in flash. `_srelocate` and `_erelocate` mark the address range in
 *    SRAM that mutable program data is copied to.
 *
 *    Tock will copy `_erelocate` - `_srelocate` bytes of data from the
 *    `_etext` pointer to the `_srelocate` pointer.
 *
 * `_szero`, `_ezero`
 *
 *    The `_szero` and `_ezero` symbols define the range of the BSS, SRAM that
 *    Tock will zero on boot.
 *
 * `_sapps`
 *
 *    The `_sapps` symbol marks the beginning of application memory in flash.
 */


SECTIONS
{
    /* STATIC ELEMENTS FOR TOCK KERNEL */
    .text :
    {
        . = ALIGN(4);
        _textstart = .;         /* Symbol expected by some MS build toolchains */
        _stext = .;         /* First of standard s,e (start/end) pair */

        /* Place vector table at the beginning of ROM.
         *
         * The first 16 entries in the ARM vector table are defined by ARM and
         * are common among all ARM chips. The remaining entries are
         * chip-specific, which Tock defines in a separate .irqs section
         *
         * http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0553a/BABIFJFG.html
         */
        KEEP(*(.vectors .vectors.*))
        KEEP(*(.irqs))

        /* .text and .rodata hold most program code and immutable constants */
        /* .gnu.linkonce hold C++ elements with vague linkage
                https://gcc.gnu.org/onlinedocs/gcc/Vague-Linkage.html */
        *(.text .text.* .gnu.linkonce.t.*)
        *(.rodata .rodata* .gnu.linkonce.r.*)

        /* C++ exception unwinding information */
        *(.ARM.extab* .gnu.linkonce.armextab.*)

        /* glue_7 and glue_7t hold helper functions emitted by the compiler to
           support interworking (linking between functions in ARM and THUMB
           mode). Note that Cortex-M's do not support ARM mode, but this is left
           here to save someone headache if they ever attempt to port Tock to a
           Cortex-A core.  */
        *(.glue_7t) *(.glue_7)


        /* Constructor and destructor sections:

           - init/fini
              Defined by ELF as sections that hold `process
              initialization/termination code`
           - {pre}{init/fini}_array_{start/end}
              Symbols used by the C runtime for initialization / termination
           - ctors/dtors
              Symbols used by the C++ runtime for initialization / termination
        */
        . = ALIGN(4);
        KEEP(*(.init))
        . = ALIGN(4);
        __preinit_array_start = .;
        KEEP (*(.preinit_array))
        __preinit_array_end = .;

        . = ALIGN(4);
        __init_array_start = .;
        KEEP (*(SORT(.init_array.*)))
        KEEP (*(.init_array))
        __init_array_end = .;

        . = ALIGN(4);
        KEEP (*crtbegin.o(.ctors))
        KEEP (*(EXCLUDE_FILE (*crtend.o) .ctors))
        KEEP (*(SORT(.ctors.*)))
        KEEP (*crtend.o(.ctors))

        . = ALIGN(4);
        KEEP(*(.fini))

        . = ALIGN(4);
        __fini_array_start = .;
        KEEP (*(.fini_array))
        KEEP (*(SORT(.fini_array.*)))
        __fini_array_end = .;

        KEEP (*crtbegin.o(.dtors))
        KEEP (*(EXCLUDE_FILE (*crtend.o) .dtors))
        KEEP (*(SORT(.dtors.*)))
        KEEP (*crtend.o(.dtors))
        /* End constructor/destructor */
    } > rom


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
    } > rom
    PROVIDE_HIDDEN (__exidx_end = .);

    /* Region for on-chip kernel non-volatile storage.
     * Align on 512 bytes since that is the page size.
     * Volumes within this region are allocated with the
     * storage_volume! macro in utils.rs. */
    . = ALIGN(512);
    .storage :
    {
      _sstorage = .;
      *(.storage* storage*)
      _estorage = .;
      . = ALIGN(512);
    } > rom
    . = ALIGN(512);

    /* Mark the end of static elements */
    . = ALIGN(4);
    _etext = .;
    _textend = .;   /* alias for _etext expected by some MS toolchains */


    /* Customer configuration is most often located
     * at the end of the rom. It is conditional, and won't
     * be written if not specified in the board specific linker
     * file. */
    .ccfg : {
        KEEP(*(.ccfg))
    } > ccfg


    /* STATIC ELEMENTS FOR TOCK APPLICATIONS */
    .apps :
    {
        /* _sapps symbol used by tock to look for first application */
        . = ALIGN(4);
        _sapps = .;

        /* Optional .app sections a convenience mechanism to bundle tock
           kernel and apps into a single image */
        KEEP (*(.app.*))
    } > prog







    /* Kernel data that must be relocated. This is program data that is
     * expected to live in SRAM, but is initialized with a value. This data is
     * physically placed into flash and is copied into SRAM by Tock. The
     * symbols here will be defined with addresses in SRAM.
     *
     * Tock assumes the relocation section follows all static elements and will
     * copy (_erelocate - _srelocate) bytes from _etext to _srelocate.
     */
    .relocate : AT (_etext)
    {
        . = ALIGN(4);
        _srelocate = .;
        *(.ramfunc .ramfunc.*);
        *(.data .data.*);

        . = ALIGN(4);
        _erelocate = .;
    } > ram


   .stack (NOLOAD) :
    {
        /* Kernel stack.
         *
         * Tock places the kernel stack at the bottom of SRAM so that the
         * kernel will trigger memory fault if it exceeds its stack depth,
         * rather than silently overwriting valuable data.
         */
        . = ALIGN(8);
         _sstack = .;

         /* For GNU LD, we can just advance the location pointer (".") here to
          * reserve space for the stack. That, however, doesn't seem to work
          * for LLVM LLD. The resulting ELF has a stack section that shows the
          * correct size, but the next section (in our case .relocate) is not
          * moved down as well, instead it sits at the same address as .stack.
          * To work around this, we declare a dummy buffer and then insert it
          * here in the .stack section. This sets the stack size correctly and
          * places the .relocate section at the correct address. */
         KEEP(*(.stack_buffer))
         /*. = . + 0x1000;*/  /*This is the original method. */

         . = ALIGN(8);
         _estack = .;
    } > ram


    .sram (NOLOAD) :
    {
        /* Kernel BSS section. Memory that is expected to be initialized to
         * zero.
         *
         * Elements in this section do not contribute to the binary size. Tock
         * initialization will write zeros to the memory between _szero and
         * _ezero.
         *
         * Elements placed in the .bss and .COMMON sections are simply used to
         * measure amount of memory to zero out.
         */
        . = ALIGN(4);
        _szero = .;

        *(.bss .bss.*)
        *(COMMON)

        . = ALIGN(4);
        _ezero = .;



        /* Application Memory.
         *
         * Tock uses the remainder of SRAM for application memory.
         *
         * Currently, Tock allocates a fixed array of application memories at
         * compile-time, and that array is simply placed here. A possible
         * future enhancement may allow the kernel to parcel this memory space
         * dynamically, requiring changes to this section.
         */
        . = ALIGN(MPU_MIN_ALIGN);
        *(.app_memory)
    } > ram
}

ASSERT((_etext-_stext) + (_erelocate-_srelocate) < LENGTH(rom), "
Text plus relocations exceeds the available ROM space.");
