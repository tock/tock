Adafruit CLUE - nRF52840 Express with Bluetooth LE
==================================================

<img src="https://cdn-learn.adafruit.com/assets/assets/000/087/843/medium640/adafruit_products_Clue_top_angle.jpg?1580406577" width="35%">

The [Adafruit CLUE - nRF52840 Express with Bluetooth LE](https://www.adafruit.com/product/4500) is a
board based on the Nordic nRF52840 SoC. It includes the
following sensors:

- 9 axis inertial LSM6DS33 + LIS3MDL sensor 
- humidity and temperature sensor
- barometric sensor
- microphone
- gesture, proximity, light color and light intensity sensor

It has the same form factor with BBC:Microbit

## Getting Started

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

You will need Adafruit's nrfutil bootloader tool:

```shell
$ pip3 install --user adafruit-nrfutil
```

## Programming the Kernel

To program the kernel we use the Adafruit version of nRFUtil tool to communicate with the bootloader
on the board which then flashes the kernel. This requires that the bootloader be
active. To force the board into bootloader mode, press the button on the back of the board
twice in rapid succession. You should see the red LED pulse on and off.

At this point you should be able to simply run `make program` in this directory
to install a fresh kernel.

```
$ make program
```

You may need to specify the port like so:

```
$ make program PORT=<serial port path>
```

## Programming Applications

For now, flashing apps is only available together with the kernel

```
$ git clone https://github.com/tock/libtock-c
$ cd libtock-c/examples/blink
$ make
```

This previous step will create a TAB (`.tab` file) that normally tockloader
would use to program on the board. However, tockloader is currently not
supported.

### Userspace Resource Mapping

This table shows the mappings between resources available in userspace
and the physical elements on the CLUE nRF52480 board.

| Software Resource | Physical Element    |
|-------------------|---------------------|
| GPIO[2]           | Pad D2              |
| GPIO[3]           | Pad D3              |
| GPIO[4]           | Pad D4              |
| GPIO[6]           | Pad D6              |
| GPIO[7]           | Pad D7              |
| GPIO[8]           | Pad D8              |
| GPIO[9]           | Pad D9              |
| GPIO[10]          | Pad D10             |
| GPIO[12]          | Pad D12             |
| LED[0]            | Red LED Red         |
| LED[1]            | White LEDs          |

## Debugging

The CLUE nRF2840 board uses a virtual serial console over USB to send debugging info
from the kernel and print messages from applications. You can use whatever your
favorite serial terminal program is to view the output. Tockloader also
supports reading and writing to a serial console with `tockloader listen`.

### Kernel Panics

If the kernel or an app encounter a `panic!()`, the panic handler specified in
`io.rs` is called. This causes the kernel to stop. You will notice the red
LED starts blinking in a repeating but slightly irregular pattern. There is also
a panic print out that provides a fair bit of debugging information. That panic
output is output over the USB CDC connection and so should be visible as part
of the output of `tockloader listen`, however if your kernel panics so early
that the USB connection has not yet been established you will be unable to view
any panic output. In this case, you can modify the panic handler to instead
output panic information over the UART pins, but you will have to separately interface
with the UART pins on the board in order to observe the serial output.
