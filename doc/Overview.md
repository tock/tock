# Tock Overview

Tock is a secure, embedded operating system for Cortex-M microcontrollers.
While it could potentially be ported to other architectures, its current
design and implementation assumes a Cortex-M that has a memory protection
unit (MPU). Systems without an MPU cannot simultaneously support untrusted
processes and retain Tock's safety and security properties. The Tock
kernel and its extensions (called *capsules*) are written in Rust.

Tock can run multiple, independent untrusted processes written in
any language. The number of processes Tock can simultaneously support
is constrained by MCU flash and RAM. The Tock scheduler is preemptive and
uses a round-robin policy. Tock uses a microkernel architecture: complex
drivers and services are often implemented as untrusted processes, which
other processes, such as applications, can invoke through inter-process
commmunication (IPC).

This document gives an overview of Tock's architecture, the different
classes of code in Tock, the protection mechanisms it uses, and how this
structure is reflected in the software's directory structure.

## Tock Architecture

![Tock architecture](architecture.png)

The above Figure shows Tock's architecture. Code falls into one of three
categories: the *core kernel*, *capsules*, and *processes*.

The core kernel and capsules are both written in Rust. Rust is a
type-safe systems language; other documents discuss the language and
its implications to kernel design in greater detail, but the
key idea is that Rust code can't use memory differently than intended
(e.g., overflow buffers, forge pointers, or have pointers to dead
stack frames). Because these restrictions prevent many things that
an OS kernel has to do (such as access a peripheral that exists at a
memory address specified in a datasheet), the very small core kernel
is allowed to break them by using "unsafe" Rust code. Capsules,
however, cannot use unsafe features. This means that the core kernel
code is very small and carefully written, while new capsules added
to the kernel are safe code and so do not have to be trusted.

Processes can be written in any language. The kernel protects itself and
other processes from bad process code by using a hardware memory
protection unit (MPU). If a process tries to access memory it's not
allowed to, this triggers an exception. The kernel handles this exception
and kills the process.

The kernel provides four major system calls:

  * command: makes a call from the process into the kernel
  * subscribe: registers a callback in the process for an upcall from the kernel
  * allow: gives kernel access to memory in the process
  * yield: suspends process until after a callback is invoked

Every system call except yield is non-blocking. Commands that
might take a long time (such as sending a message over a UART)
return immediately and issue a callback when they complete.
The yield system call blocks the process until a callback
is invoked; userland code typically implements blocking
functions by invoking a command and then using yield to wait
until the callback completes.

The command, subscribe, and allow system calls all take a driver
ID as their first parameter. This indicates which driver in the
kernel that system call is intended for. Drivers are capsules that
implement the system call 

## Tock Memory Map

Tock is intended to run on Cortex-M microcontrollers, which have
non-volatile flash memory (for code) and RAM (for stack and data) in a
single address space. While the Cortex-M architecture specifies a
high-level layout of the address space, the exact layout of Tock can
differ from board to board. Most boards simply define the beginning and
end of flash and SRAM in their `layout.ld` file and then include the
[generic Tock memory map](../boards/kernel_layout.ld).

Tock's memory has three major regions: kernel code, process code, and
RAM: For the SAM4L, these are laid out as follows. This allocation assumes
the SAM4L which has 512kB of flash and 64kB of RAM:

| Region  | Start Address | Length |
| ------- | ------------- | ------ |
| kernel  | 0x00000000    |  192kB |
| process | 0x00030000    |  320kB |
| RAM     | 0x20000000    |   64kB |

So Tock allocates 192kB of flash for kernel code and 320kB of flash for
appliation code.

### Kernel code

The kernel code is split into two major regions, `.text` which holds the
vector table, program code, initialization routines, and other read-only data.
This section is written to the beginning of flash.

This is immediately followed by the `.relocate` region, which holds values the
need to exist in SRAM, but have non-zero initial values that Tock copies from
flash to SRAM as part of its initialization (see [Startup](Startup.md)).

### Process code

Processes can either be statically compiled into a Tock image,
or dynamically loaded onto the microcontroller. The symbol `_sapps`
denotes the start of the process code section. The process code section
has one or more process code blocks in it; the kernel defines a static
limit for how many processes can be supported. Imix, for example, currently
supports two processes. This is a static number so that the kernel does
not have to dynamically allocate memory.

The first word of a process code block is the length of the block. So the
first word of the process code section is the length of the code of the
first process (including this size field). If the length is zero, there is
no process. So on boot, the kernel checks the length at `_sapps`, if it is
non-zero, loads the process code there, then checks the length at `_sapps`
plus the length of the first process. This continues until it finds a
length of zero or it reaches the maximum number of processes.

| contents |       size     |
| -------- | ---------------|
|  length  | (4 bytes)      |
|   code   | (length bytes) |
|  length  | (4 bytes)      |
|   code   | (length bytes) |

Note that this means a linker script needs to set the first unused word
of the application code region to 0; otherwise, if there is uncleared flash
memory the kernel might conclude there are applications there.

### RAM

RAM contains four major regions:

* kernel data (initialized memory, copied from flash at boot),
* kernel BSS (uninitialized memory, zeroed at boot),
* the kernel stack,
* process memory.

## Tock Directory Structure

Tock has seven principal code directories.

The *arch* directory stores architecture-specific code. I.e., code that
is Cortex-M0 and Cortex-M4 specific. This includes code for performing
context switches and making system calls (trapping from user code to
kernel code).

The *boards* directory contains code for specific Tock platforms, such as
the imix, the Firestorm, and the nrf51dk. This is typically the structure
that defines all of the capsules the kernel has, the code to configure the
MCU's IO pins into the proper states, initializing the kernel and loading
processes. The principal file in this directory is `main.rs`, and the
principal initialization function is `reset_handler` (which executes
when the MCU resets). The board code also defines how system call device
identifiers map to capsules, in the `with_driver` function.

The *capsules* directory contains MCU-independent kernel extensions that
can build on top of chip-specific implementations of particular peripherals.
Some capsules provide system calls. For example, the `spi` module in capsules
builds on top of a chip's SPI implementation to provide system calls on
top of it.

The *chips* directory contains microcontroller-specific code, such as the
implementations of SPI, I2C, GPIO, UART, and other microcontroller-specific
code. The distinction between chips and boards is the difference between
a microcontroller and a full platform. For example, many microcontrollers
have multiple UARTs. Which UART is the principal way to communicate with
Tock, or which is used to control another chip, is defined by how the chip
is placed on board and which pins are exposed. So a chip provides the UART
implementation, but a board defines which UART is used for what.

The *extern* directory contains external code that is not part of the Tock
operating system, such as the Rust compiler.

The *kernel* directory contains microcontroller-independent kernel code,
such as the scheduler, processes, and memory management. This directory
and arch are were where all core kernel code reside.

The *tools* directory are associated tools to help in compilation and
code maintenance, such as checking code formatting, converting binaries,
and build scripts.

The *userland* directory contains process code, including example
applications, userland drivers, and the userland system call functions
that translate friendly API calls such as `led_on(int led_num)` into
underlying system calls such as `command(DRIVER_NUM_LEDS, 0, led_num)`.










