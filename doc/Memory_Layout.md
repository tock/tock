Memory Layout
=============

This document describes how the memory in Tock is used for the
kernel, applications, and supporting state.

<!-- npm i -g markdown-toc; markdown-toc -i Memory_Layout.md -->

<!-- toc -->

- [Kernel code](#kernel-code)
- [Process code](#process-code)
- [RAM](#ram)
- [Hardware Implementations](#hardware-implementations)
  * [SAM4L](#sam4l)
    + [Flash](#flash)
    + [RAM](#ram-1)

<!-- tocstop -->

Tock is intended to run on Cortex-M microcontrollers, which have
non-volatile flash memory (for code) and RAM (for stack and data) in a
single address space. While the Cortex-M architecture specifies a
high-level layout of the address space, the exact layout of Tock can
differ from board to board. Most boards simply define the beginning and
end of flash and SRAM in their `layout.ld` file and then include the
[generic Tock memory map](../boards/kernel_layout.ld).

## Kernel code

The kernel code is split into two major regions, `.text` which holds the
vector table, program code, initialization routines, and other read-only data.
This section is written to the beginning of flash.

This is immediately followed by the `.relocate` region, which holds values the
need to exist in SRAM, but have non-zero initial values that Tock copies from
flash to SRAM as part of its initialization (see [Startup](Startup.md)).

## Process code

Processes can either be statically compiled into a Tock image,
or dynamically loaded onto the microcontroller. The symbol `_sapps`
denotes the start of the process code section. The process code section
has one or more process code blocks in it; the kernel defines a static
limit for how many processes can be supported. Imix, for example, currently
supports two processes. This is a static number so that the kernel does
not have to dynamically allocate memory.

Each process starts with a Tock Binary Format (TBF) header and then the actual
application binary. Processes are placed continuously in flash. The end of the
valid processes are denoted by an invalid TBF header. Typically the flash page
after the last valid process is set to all 0x00 or 0xFF.

## RAM

RAM contains four major regions:

* kernel data (initialized memory, copied from flash at boot),
* kernel BSS (uninitialized memory, zeroed at boot),
* the kernel stack,
* process memory.


## Hardware Implementations

Here are how things are laid out in practice.

### SAM4L

The SAM4L is used on the Hail and Imix platforms, as well as others.

#### Flash

| Address Range   | Length (bytes) | Content    | Description                                                                                      |
|-----------------|----------------|------------|--------------------------------------------------------------------------------------------------|
| 0x0-3FF         | 1024           | Bootloader | Reserved flash for the bootloader.  Likely the vector table.                                     |
| 0x400-0x5FF     | 512            | Flags      | Reserved space for flags. If the bootloader is present, the first 14 bytes are "TOCKBOOTLOADER". |
| 0x600-0x9FF     | 1024           | Attributes | Up to 16 key-value pairs of attributes that describe the board and the software running on it.   |
| 0xA00-0xFFFF    | 61.5k          | Bootloader | The software bootloader provides non-JTAG methods of programming the kernel and applications.    |
| 0x10000-0x2FFFF | 128k           | Kernel     | Flash space for the kernel.                                                                      |
| 0x30000-0x7FFFF | 320k           | Apps       | Flash space for applications.                                                                    |

#### RAM

| Address Range         | Length (bytes) | Content            | Description                                                                                       |
|-----------------------|----------------|--------------------|---------------------------------------------------------------------------------------------------|
| 0x20000000-0x2000FFFF | 64k            | Kernel and app RAM | The kernel links with all of the RAM, and then allocates a buffer internally for application use. |
