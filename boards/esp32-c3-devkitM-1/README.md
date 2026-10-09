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
ESP-ROM:esp32c3-eco7-20230720
Build:Jul 20 2023
rst:0x1 (POWERON),boot:0xc (SPI_FAST_FLASH_BOOT)
flash: QIO at 80 MHz
ESP32-C3 initialisation complete.
Entering main loop.
```

```shell
screen /dev/ttyUSB0  115200
```

## Flash Mode

The kernel runs from external SPI flash using XIP (execute in place). To speed
up accesses to this flash, Tock re-configures the SPI bus to this chip on boot
from 20 MHz to 80 MHz.

Most ESP32-C3 modules further support Quad-I/O mode for the SPI flash, which
reads 4 bits per clock-cycle, and allows for even faster flash accesses.
However, this depends on the WP# and HOLD# pins being connected properly, and
the SPI flash supporting this mode. For such modules, Quad-I/O must be enabled
by setting the QE bit in the flash status register in the SPI flash.

To enable Quad-I/O mode, first inspect the current flash status register
contents. You'll want to modify those in the next step:

```shell
esptool.py --chip esp32c3 read_flash_status --bytes 2
```

You'll need to take the returned value for the next step. Bit 9 is the QE bit,
and governs whether Tock will attempt to use the SPI flash chip in DIO or QIO
mode. For example, this may print 0x0000 for !QE (DIO-mode) or 0x0200 for QE
(QIO-mode). Be sure to keep any bits other than bit 9 identical.

Then enable QIO mode by writing the value that you read with bit 9 set. For
example, if the above returned `0x0000`, you can write `0x0200`. Make sure that
you leave other bits untouched.

```shell
esptool.py --chip esp32c3 write_flash_status --non-volatile --bytes 2 <$VAL | (1 << 9)>
```

You can disable QIO mode by clearing bit 9. For example, if you previously wrote
`0x0200`, you can switch back to DIO by writing `0x0000`.

```shell
esptool.py --chip esp32c3 write_flash_status --non-volatile --bytes 2 <$VAL & ~(1 << 9)>
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
