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
