# ESP32-C3 Board

ESP32-C3 is a system on a chip that integrates the following features:

- Wi-Fi (2.4 GHz band)
- Bluetooth Low Energy
- High performance 32-bit RISC-V-ish single-core processor
- Multiple peripherals
- Built-in security hardware

Powered by 40 nm technology, ESP32-C3 provides a robust, highly integrated
platform, which helps meet the continuous demands for efficient power usage,
compact design, security, high performance, and reliability.

## Setup

Install the ESP tool:

```shell
# macOS
brew install esptool
```

of from source:

```shell
git clone https://github.com/espressif/esptool.git
cd esptool
pip install --user -e .
```

The first time you are installing Tock you probably want to erase the
flash first. This can be done with:

```shell
esptool.py --chip esp32c3 erase_flash
```

After that you can run:

```shell
make init
make flash
```

To install Tock.

You can then connect to the serial device exposed from the USB header on the
board.

You can use the `RST` button on the board to reset Tock. You should see
something similar to:

```text
ESP-ROM:esp32c3-20200918
Build:Sep 18 2020
rst:0x1 (POWERON),boot:0xc (SPI_FAST_FLASH_BOOT)
SPIWP:0xee
mode:DIO, clock div:1
load:0x40380000,len:0xd15c
load:0x4038d15c,len:0xccc
load:0x00000000,len:0x21a0
load:0x42000000,len:0x24
SHA-256 comparison failed:
Calculated: 63cf02fff6c0e3f60d140721bbd74adf0072c368b3bfafb6d4195511a55ba8c9
Expected: f4494a64f93940e1bb4d0edce76f041e6a411337097c697b254b784af1e2bcd5
Attempting to boot anyway...
entry 0x40380000
ESP32-C3 initialisation complete.
Entering main loop.
```

```shell
screen /dev/ttyUSB0  115200
```

## Building and Flashing Applications

Apps are built out-of-tree, for example:

```bash
$ cd libtock-c/examples/<app>
$ make
```

To "flash" an app, we first write it to a binary file that is combined with the
kernel which we will write in its entirety to the board.

```bash
$ tockloader install --local-board
```

Then to flash the kernel with the app:

```
$ cd tock/boards/esp32-c3-devkitM-1
$ make flash
```

## JTAG Debugging

In order to use JTAG debugging you first need to build a fork of OpenOCD

```shell
git clone https://github.com/espressif/openocd-esp32
cd openocd-esp32
./bootstrap
./configure --disable-werror
make -j8
```

Note that the connection can be unreliable, commit
5b67ad1b15938f524f133afb8ef652c990f570eb seems to work though.

Then connect an [FTDI C232HM](https://ftdichip.com/products/c232hm-ddhsl-0-2/)
cable as described here:
https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/api-guides/jtag-debugging/configure-other-jtag.html.

Make sure that the board is powered by the JTAG and that the usual USB
connection is unplugged when doing JTAG debugging.

Then run OpenOCD

```shell
./src/openocd -s tcl -f tcl/board/esp32c3-ftdi.cfg
```

Then connect from GDB

```shell
target remote :3333
set remote hardware-watchpoint-limit 2
set mem inaccessible-by-default off
```
