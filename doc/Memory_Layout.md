Memory Layout
=============

This document describes how the memory in Tock is used for the
kernel, applications, and supporting state.

### SAM4L Flash

| Address Range   | Length (bytes) | Content    | Description                                                                                      |
|-----------------|----------------|------------|--------------------------------------------------------------------------------------------------|
| 0x0-3FF         | 1024           | Bootloader | Reserved flash for the bootloader.  Likely the vector table.                                     |
| 0x400-0x5FF     | 512            | Flags      | Reserved space for flags. If the bootloader is present, the first 14 bytes are "TOCKBOOTLOADER". |
| 0x600-0x9FF     | 1024           | Attributes | Up to 16 key-value pairs of attributes that describe the board and the software running on it.   |
| 0xA00-0xFFFF    | 61.5k          | Bootloader | The software bootloader provides non-JTAG methods of programming the kernel and applications.    |
| 0x10000-0x2FFFF | 128k           | Kernel     | Flash space for the kernel.                                                                      |
| 0x30000-0x7FFFF | 320k           | Apps       | Flash space for applications.                                                                    |

