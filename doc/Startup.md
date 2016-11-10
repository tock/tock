# Tock Startup & Memory Map

This document walks through how all of the components of Tock start up. It's
broken into four major sections

  - [Bootup](#bootup): From vector table to kernel entry
  - [Kernel Initialization](#kernel-initialization): What starts in Tock's core
    and how it happens
  - [Capsule Initialization](#capsule-initialization): Capsules, essentially
    built-in drivers, are then loaded by the kernel
  - [Application Startup](#application-startup): How applications are loaded
    into memory, started, and scheduled


## Bootup

While Tock initially focuses on Cortex-M processors, the only real assumption
it makes is the ability to place a function at an entry point on bootup.

### Vector Table and IRQ table

When a microcontroller boots (or resets, or services an interrupt) it loads an
address for a function from a table indexed by interrupt type known as the
_vector table_.  The location of the vector table in memory is chip-specific,
thus it is placed in a special section for linking.

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

### Reset Handler

As chip-specific reset operations are handled by the chip itself, Tock has a
unique `reset_handler` function defined in `boards/<board>/src/main.rs` for
each board to handle any platform-specific reset obligations.


## Kernel Initialization

## Capsule Initialization

## Application Startup

