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

+ ### Console paths: serial vs. VGA

By default the board prints all kernel output over **COM 1** and QEMU opens a
terminal window for you.  
If you prefer the 80 × 25 **VGA text console** instead, just pass
QEMU’s headless flag:

To run the kernel use `cargo run`.

To boot QEMU without a display use `cargo run -- -display none`.

+ ### Internally the board picks the console path at boot:

`vga::new_text_console()` initialises the VGA registers and maps the 0xB8000 text buffer.

Two UART muxes are created - one for COM 1 and one backed by the new VGA text driver.

The debug writer is wired to the VGA mux, while the interactive
ProcessConsole stays on COM 1 until a keyboard driver lands.



