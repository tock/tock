MakePython nRF52840
===================

<img src="https://www.makerfabs.com/media/catalog/product/cache/5082619e83af502b1cf28572733576a0/m/a/makepython_nrf52840-2.jpg" width="35%">

The [MakePython nRF52840](https://www.makerfabs.com/makepython-nrf52840.html) is
a development board with the Nordic nRF52840 SoC and a 128 x 64 pixel OLED
display.


## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md).

The MakePython nRF52840 is designed to be programmed using an external JLink
programmer. We would like to avoid this requirement, so we use the [Tock
Bootloader](https://github.com/tock/tock-bootloader) which allows us to program
the board over the UART connection. However, we still require the programmer one
time to flash the bootloader.

To flash the bootloader we must connect a JLink programmer. The easiest way is
to use an nRF52840dk board.

### Connect the nRF52840dk to the MakePython-nRF52840

First we jumper the board as shown with the following pin mappings

| nRF52840dk | MakePython-nRF52840 |
|------------|---------------------|
| GND        | GND                 |
| SWD SEL    | +3V3                |
| SWD CLK    | SWDCLK              |
| SWD IO     | SWDDIO              |

Make sure _both_ the nRF52840dk board and the MakePython-nRF52840 board are
attached to your computer via two USB connections.

Then:

```
make flash-bootloader
```

This will use JLinkExe to flash the bootloader using the nRF52840dk's onboard
jtag hardware.

### Using the Bootloader

The bootloader activates when the reset button is pressed twice in quick
succession. The green LED will stay on when the bootloader is active.

Once the bootloader is installed tockloader will work as expected.
