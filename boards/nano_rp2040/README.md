Arduino Nano RP2040 Copnnect
============================

<img src="https://store-cdn.arduino.cc/uni/catalog/product/cache/1/image/520x330/604a3538c15e081937dbfbd20aa60aad/a/b/abx00053_00.default.jpg" width="35%">

The [Arduino Nano RP2040 Connect](https://docs.arduino.cc/hardware/nano-rp2040-connect) is an Arduino Nano
board built using the Raspberry Pi Foundation's RP2040 chip.

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md)

## Flashing the kernel

The Arduino Nano RP2040 Connect can be programmed using its bootloader, which requires an UF2 file.

`cd` into `boards/nano_rp2040` directory and run:

```bash
$ make

(or)

$ make debug
```

## Flashing app

Apps are built out-of-tree. Once an app is built, you can add the path to it in the Makefile (APP variable), then run:
```bash
$ make program
```

This will generate a new ELF file that can be deployed on the Raspberry Pi Pico via gdb and OpenOCD as described in the [section above](#flash-the-tock-kernel).
