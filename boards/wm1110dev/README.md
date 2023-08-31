Wio WM1110 Development Board
===================

<img src="https://media-cdn.seeedstudio.com/media/catalog/product/cache/bb49d3ec4ee05b6f018e93f896b8a25d/1/-/1-114993082-wio-wm1110-dev-kit-45font.jpg" width="35%">

The [Wio WM1110 Development
Board](https://www.seeedstudio.com/Wio-WM1110-Dev-Kit-p-5677.html) is a
multiradio location board for LoRa and location services based on the Nordic
nRF52840 SoC. The includes the following hardware:

- LIS3DHTR: 3-Axis cccelerometer
- SHT41: Temperature and humidity sensor
- GNSS
- WiFi AP Scanning
- Semtech LR1110


## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md).

The WM1110 must be initially programmed using an external JLink programmer. We
first flash the [Tock Bootloader](https://github.com/tock/tock-bootloader) so we
only require the programmer once.

To flash the bootloader we must connect a JLink programmer. The easiest way is
to use an nRF52840dk board.

### Connect the nRF52840dk to the WM1110-dev

First we jumper the board as shown here:
https://devzone.nordicsemi.com/f/nordic-q-a/97159/nrf52-dk-v3-0-0-debug-out-for-programming-external-board-can-t-work

Then:

```
make flash-bootloader
```

### Using the Bootloader

The bootloader activates when the reset button is pressed twice in quick
succession. The green LED will stay on when the bootloader is active.

Once the bootloader is installed tockloader will work as expected.
