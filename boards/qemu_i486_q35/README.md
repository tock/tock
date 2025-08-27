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
sudo apt install qemu-system-x86
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

By default `cargo run` launches QEMU exactly as the build script spells out:
`qemu-system-i386 -cpu 486 -machine q35 -net none -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -kernel`.  
That gives you **two views**:

* a VGA window where all `debug!()` output from the kernel is shown, and
* the serial terminal on the right (stdout) that hosts the interactive
  `ProcessConsole` shell.

If you instead run `cargo run -- -display none`
the extra `-display none` flag tells QEMU to skip creating the VGA window.
The kernel still programs the VGA hardware, but you will not see that screen;
only the serial terminal (ProcessConsole) remains visible. 

Regardless of the flag, ProcessConsole is always on the serial port, while kernel debug messages
are routed to VGA whenever a display is present.




