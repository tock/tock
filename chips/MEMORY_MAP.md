Tock Memory Map
===============

Most platforms should be able to use the common memory map provided with Tock
by simply defining regions of memory for the kernel (ROM), user applications
(PROG), and SRAM (RAM).

Example platform memory map:

    /* Memory Spaces Definitions, 448K flash, 64K ram */
    ROM_ORIGIN  = 0x00010000;
    ROM_LENGTH  = 0x00020000;
    PROG_ORIGIN = 0x00030000;
    PROG_LENGTH = 0x00040000;
    RAM_ORIGIN  = 0x20000000;
    RAM_LENGTH  = 0x00010000;

    MPU_MIN_ALIGN = 8K;

    INCLUDE ../kernel_layout.ld


Tock's default memory map should pull in all of the standard sections expected
by C and C++ libraries (`.ctors`, `.crt*`, etc) and ARM exception unwinding
(`.ARM.exidx*`, etc).

Special Sections in Tock's Memory Map
-------------------------------------

### `.vectors`

Tock's default memory map assumes that the chip's vector table should be placed
at `ROM_ORIGIN`. It also assumes that chip code has defined an array (or
equivalent structure) to be placed in a linkage section called `.vectors`, like
this:

```rust
// Rust Example:
#[link_section=".vectors"]
#[no_mangle]                        // Ensures that the symbol is kept until the final binary
pub static BASE_VECTORS: [unsafe extern fn(); 16] = [
    _estack,                        // Initial stack pointer value
    tock_kernel_reset_handler,      // Tock's reset handler function
    /* NMI */ unhandled_interrupt,  // Generic handler function
    ...
```

```c
// C Example:
__attribute__ ((section(".vectors")))
interrupt_function_t interrupt_table[] = {
	(interrupt_function_t) (&_estack),
	tock_kernel_reset_handler,
	NMI_Handler,
	...
```

### `.app_memory`

Tock allocates a region of memory in SRAM for processes, like this:

```rust
#[link_section = ".app_memory"]
static mut MEMORIES: [[u8; 8192]; NUM_PROCS] = [[0; 8192]; NUM_PROCS];
```

Linkers must place this region into SRAM.


Custom Memory Maps
------------------

Chip and/or platform authors are welcome to create custom memory maps. In
addition to the sections mentioned above, there are a few symbols that Tock
expects the linker file to define:

### `_etext`, `_srelocate`, `_erelocate`

The `_etext` symbol marks the end of data stored in flash that should stay in
flash. `_srelocate` and `_erelocate` mark the address range in SRAM that mutable
program data is copied to.

Tock will copy `_erelocate` - `_srelocate` bytes of data from the `_etext`
pointer to the `_srelocate` pointer.

### `_szero`, `_ezero`

The `_szero` and `_ezero` symbols define the range of the BSS, SRAM that Tock
will zero on boot.

### `_sapps`

The `_sapps` symbol marks the beginning of application memory in flash.

