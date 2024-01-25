Apollo3 Boards
==============

This directory contains all of the Apollo3 boards supported by Tock

 * ambiq - Generic tools for flashing binaries
 * redboard_artemis_atp [SparkFun RedBoard Artemis ATP](https://www.sparkfun.com/products/15442)
 * redboard_artemis_nano [SparkFun RedBoard Artemis Nano](https://www.sparkfun.com/products/15443)
 * lora_things_plus [SparkFun LoRa Thing Plus - expLoRaBLE](https://www.sparkfun.com/products/17506)

## Hardware differences

All of the boards use the same SoC, so the Tock board files are overall very
similar and can actually be used interchangably for basic operations.

The main difference between them is what is broken out via the boards. For
example the Redboard Artemis ATP uses IOM4 for the Qwiic connector while
the Redboard Artemis Nano uses IOM2.

The GPIO breakouts are also a little different.

The LoRa Thing Plus also sets up the SX1262 radio, which the other boards
don't have.

## Configuration differences

The other difference is Tock configurations. The boards are configured
slightly differently to show a range of different options.

### I2C

The Redboard Artemis ATP doesn't setup a `MuxI2C`, like the other boards,
instead it creates a `I2CMasterSlaveDriver`.

A `I2CMasterSlaveDriver` can also be setup on the Redboard Artemis Nano
instead of a `MuxI2C` as it exposes the correct pins, but it isn't by default.
