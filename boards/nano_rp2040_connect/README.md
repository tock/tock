Arduino Nano RP2040 Connect
===========================

<img src="https://store-cdn.arduino.cc/uni/catalog/product/cache/1/image/520x330/604a3538c15e081937dbfbd20aa60aad/a/b/abx00053_00.default.jpg" width="35%">

The [Arduino Nano RP2040 Connect](https://docs.arduino.cc/hardware/nano-rp2040-connect) is an Arduino Nano
board built using the Raspberry Pi Foundation's RP2040 chip.

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md)

## Installing elf2uf2

The Nano RP2040 uses UF2 files for flashing. Tock compiles to an ELF file.
The `elf2uf2` utility is needed to transform the Tock ELF file into an UF2 file.

To install `elf2uf2`, run the commands:

```bash
$ git clone https://github.com/raspberrypi/pico-sdk
$ mkdir build
$ cd mkdir
$ cmake ..
$ cd tools/elf2uf2
$ make
$ sudo cp elf2uf2 /usr/local/bin
```

## Flashing the kernel

The Arduino Nano RP2040 Connect can be programmed using its bootloader, which requires an UF2 file.

### Enter BOOTSEL mode

To flash the Nano RP2040, it needs to be put into BOOTSEL mode. This will mount
a flash drive that allows one to copy a UF2 file. While the offical 
documentation states that bouble pressing the on-board button enter this mode,
this seems to work only while running Arduino's original software.

If double tapping the button does not enter BOOTSEL mode (the flash drive is not mounted),
the device can be [forced into BOOTSEL mode using a jumper wire](https://docs.arduino.cc/tutorials/nano-rp2040-connect/rp2040-01-technical-reference#forcing-bootloader).

1. Disconenct the board from USB
2. Connect the GND pin with the REC pin
3. Connect the board to USB
4. Wait for the flash drive to mount
5. Disconnect the board from USB (*very important*)

`cd` into `boards/nano_rp2040_connect` directory and run:

```bash
$ make flash

(or)

$ make flash-debug
```

> Note: The Makefile provides the BOOTSEL_FOLDER variable that points towards the mount point of
> the Nano RP2040 flash drive. By default, this is located in `/media/$(USER)/RP2040`. This might
> be different on several systems, make sure to adjust it.

## Flashing app

Enter BOOTSEL mode.

Apps are built out-of-tree. Once an app is built, you can add the path to it in the Makefile (APP variable), then run:
```bash
$ APP="<path to app's tbf file>" make program
```

## Serial Interface

Tock for Nano RP2040 does not yet support USB. The serial console is using UART0, 
meaning that a [USB TTL adapter](https://www.adafruit.com/product/954) is needed to interface the board.
