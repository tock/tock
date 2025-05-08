QEMU i486 Q35 PC Port
=====================

This port provides Tock for x86 i486 Q35 simulated processor.

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md)

## Software Requirements

### QEMU System x86 (qemu-system-i386)

To install QEMU follow these steps:

**Linux**
```bash
sudo apt update
sudo apt install qemu-system-i386
```

**MacOS**
```bash
brew install qemu
```

**Windows**

Download QEMU for Windows from the official site: https://www.qemu.org/download/ and run the installer.


### GNU `objcopy` for x86 (not the one provided by LLVM)

> NOTE: `rust-objcopy` does not work, it does not rearrange the ELF sections so that
>       the Multiboot header is in the right position.

## Running the kernel

To run the kernel use `make run`.
