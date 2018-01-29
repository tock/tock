Tock Startup
============

This document walks through how all of the components of Tock start up.

<!-- npm i -g markdown-toc; markdown-toc -i Memory_Layout.md -->

<!-- toc -->

- [Optional Bootloader](#optional-bootloader)
- [Tock Vector Table and IRQ table](#tock-vector-table-and-irq-table)
- [Reset Handler](#reset-handler)
  * [Memory Initialization](#memory-initialization)
  * [MCU Setup](#mcu-setup)
  * [Capsule Initialization](#capsule-initialization)
- [Application Startup](#application-startup)
- [Scheduler Execution](#scheduler-execution)

<!-- tocstop -->

When a microcontroller boots (or resets, or services an interrupt) it loads an
address for a function from a table indexed by interrupt type known as the
_vector table_. The location of the vector table in memory is chip-specific,
thus it is placed in a special section for linking.

Cortex-M microcontrollers expect a vector table to be at address 0x00000000.
This can either be a software bootloader or the Tock kernel itself.

## Optional Bootloader

Many Tock boards (including Hail and imix) use a software bootloader that
executes when the MCU first boots. The bootloader provides a way to talk to the
chip over serial and to load new code, as well as potentially other
administrative tasks. When the bootloader has finished, it tells the MCU that
the vector table has moved (to a known address), and then jumps to a new
address.

## Tock Vector Table and IRQ table

Tock splits the vector table into two sections, `.vectors` which hold the first
16 entries, common to all ARM cores, and `.irqs`, which is appended to the end
and holds chip-specific interrupts.

In the source code then, the vector table will appear as an array that is
marked to be placed into the `.vectors` section.

In Rust, a vector table will look something like this:
```rust
#[link_section=".vectors"]
#[no_mangle] // Ensures that the symbol is kept until the final binary
pub static BASE_VECTORS: [unsafe extern fn(); 16] = [
    _estack,                        // Initial stack pointer value
    tock_kernel_reset_handler,      // Tock's reset handler function
    /* NMI */ unhandled_interrupt,  // Generic handler function
    ...
```

In C, a vector table will look something like this:

```c
__attribute__ ((section(".vectors")))
interrupt_function_t interrupt_table[] = {
	(interrupt_function_t) (&_estack),
	tock_kernel_reset_handler,
	NMI_Handler,
```

At the time of this writing (October 2016), the `sam4l` defines its vector table
in `lib.rs` as a series of `.vectors` sections that are concatenated during
linking into one table; the `nrf51` defines its vector table in `crt1.c`.

## Reset Handler

On boot, the MCU calls the reset handler function defined in vector table. In
Tock, the implementation of the reset handler function is platform-specific and
defined in `boards/<board>/src/main.rs` for each board.

### Memory Initialization

The first operation the reset handler does is setup the kernel's memory by
copying it from flash. For the SAM4L, this is in the `init()` function in
`chips/sam4l/src/lib.rs`.

### MCU Setup

Any normal MCU initialization is typically handled next. This includes things
like enabling the correct clocks or setting up DMA channels.

### Capsule Initialization

The board then initializes all of the capsules the kernel intends to support. In
the common case, this only includes "chaining" callbacks between the underlying
hardware drivers and the various capsules that use them. Sometimes, however, an
additional "setup" function is required (often called `.initialize()`), and
would be called inside of this reset handler at this point.

## Application Startup

Once the kernel components have been setup and initialized, the applications
must be loaded. This procedure essentially iterates over the processes stored in
flash, extracts and validates their Tock Binary Format header, and adds them to
an internal array of process structs.

An example version of this loop is in `kernel/src/process.rs` as the
`load_processes()` function. After setting up pointers, it tries to create a
process from the starting address in flash and with a given amount of memory
remaining. If the header is validated, it tries to load the process into memory
and initialize all of the bookeeping in the kernel associated with the process.
This can fail if the process needs more memory than is available on the chip. As
a part of this load process, the kernel can also perform PIC fixups for the
process if it was requested in the TBF header. If the process is successfully
loaded the kernel importantly notes the address of the application's entry
function which is called when the process is started.

The load process loop ends when the kernel runs out of statically allocated
memory to store processes in, available RAM for processes, or there is an
invalid TBF header in flash.

## Scheduler Execution

The final thing that the reset handler must do is call `kernel::main()`. This
starts the Tock scheduler and the main operation of the kernel.
