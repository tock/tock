SparkFun LoRa Thing Plus - expLoRaBLE
=====================================

## Board features

 - 21 Multifunctional GPIO
 - Thing Plus (or Feather) Form-Factor:
 - USB-C Connector
 - 2-pin JST Connector for a LiPo Battery (not included)
 - 4-pin JST Qwiic Connector
 - LoRa
 - BLE

For more details [visit the SparkFun
website](https://www.sparkfun.com/products/17506).

## Flashing the kernel

The kernel can be programmed using the Ambiq python scrips. `cd` into `boards/apollo3/lora_things_plus/`
directory and run:

```shell
$ make flash

(or)

$ make flash-debug
```

This will flash Tock onto the board via the /dev/ttyUSB0 port. If you would like to use a different port you can specify it from the `PORT` variable.

```bash
$ PORT=/dev/ttyUSB2 make flash
```

This will flash Tock over the SparkFun Variable Loader (SVL) using the Ambiq loader.
The SVL can always be re-flashed if you want to.


## Debugging the board

The SparkFun LoRa Thing Plus exposes JTAG via the small headers in the middle of
the board. See the [SparkFun hookup guide](https://learn.sparkfun.com/tutorials/sparkfun-explorable-hookup-guide/all) for a picture of this.

SparkFun sell accessories you can use to connecting to this. It appears
something like the J-Link BASE will work, but that hasn't been tested by Tock.

Instead, Tock has tested debugging with the [Black Magic Probe](https://black-magic.org/).
The Black Magic Probe (BMP) is an easy to use, mostly plug and play, JTAG/SWD debugger
for embedded microcontrollers.

In order to debug with the BMP, first connect the 2x5 SWD cable to the board
and the BMP.

Then power on both boards.

Fire up an ARM GDB instance and attach to the BMP with:

```
target extended-remote /dev/ttyACM0
monitor swdp_scan
attach 1
```

You can then use GDB to debug the board

## Using LoRa with the board

There is a work in progress port of a userspace LoRa library. This is based on
the Semtech library. You can find the code for this at:
https://github.com/alistair23/LoRaMac-node

You can build the application by running the following in the `LoRaMac-node`
directory:

```shell
$ mkdir build
$ cd build
$ cmake -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_TOOLCHAIN_FILE="../cmake/toolchain-arm-none-eabi.cmake" \
        -DAPPLICATION="ping-pong" \
        -DMODULATION="LORA" \
        -DREGION_EU868="OFF" \
        -DREGION_US915="OFF" \
        -DREGION_CN779="OFF" \
        -DREGION_EU433="OFF" \
        -DREGION_AU915="ON" \
        -DREGION_AS923="OFF" \
        -DREGION_CN470="OFF" \
        -DREGION_KR920="OFF" \
        -DREGION_IN865="OFF" \
        -DREGION_RU864="OFF" \
        -DBOARD="Tock" \
        -DUSE_RADIO_DEBUG="ON" ..
$ make
$ elf2tab -n ping-pong --stack 2048 --app-heap 1024 --kernel-heap 1024 --kernel-major 2 --kernel-minor 1 -v ./src/apps/ping-pong/ping-pong
```

Then in the Tock repo you can flash the app with:

```shell
$ make flash; APP=LoRaMac-node/build/src/apps/ping-pong//ping-pong.tbf make flash-app
```
