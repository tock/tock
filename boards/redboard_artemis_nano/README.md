SparkFun RedBoard Artemis Nano
==============================

## Board features

 - 17 GPIO - all interrupt capable
 - 8 ADC channels with 14-bit precision
 - 17 PWM channels
 - 2 UARTs
 - 4 I2C buses
 - 2 SPI buses
 - PDM Digital Microphone
 - Qwiic Connector

For more details [visit the SparkFun
website](https://www.sparkfun.com/products/15443).

## Flashing the kernel

The kernel can be programmed using the Ambiq python scrips. `cd` into `boards/sparkfun_redboard_artemis_nano`
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
