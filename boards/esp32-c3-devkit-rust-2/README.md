# ESP32-C3-DevKit-RUST-2 Board

<img src="https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c3/_images/esp32-c3-devkit-rust-2-block-diagram.png" width="35%">

The ESP32-C3-DevKit-RUST-2 board uses the ESP32-C3-MINI-1 module and includes:

- Wi-Fi (2.4 GHz band)
- Bluetooth Low Energy
- Temperature Sensor
- Humidity Sensor
- Accelerometer
- Gyroscope

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

To install the Tock kernel:

```shell
make init
make install
```

To install Tock.

You can then connect to the serial device exposed over the same USB Type-C
port used for flashing.

```text
$ tockloader listen

ESP-ROM:esp32c3-eco7-20230720
Build:Jul 20 2023
rst:0x15 (USB_UART_CHIP_RESET),boot:0xc (SPI_FAST_FLASH_BOOT)
Saved PC:0x42000a84
flash: QIO at 80 MHz
ESP32-C3 initialisation complete.
Entering main loop.
tock$
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
$ cd tock/boards/esp32-c3-devkit-rust-2
$ make install
```
