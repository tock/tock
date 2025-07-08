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

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md)

You will need Adafruit's nrfutil bootloader tool:

```shell
$ pip3 install --user adafruit-nrfutil
```

## Bootloader

Tock uses [Tock Bootloader](https://github.com/tock/tock-bootloader) to program devices.

The CLUE nRF52840 is shipped with the Adafruit nRF52 bootloader.

As the board has no integrated debugger, tock-bootloader will be installed on top
of the original bootloader. Flashing a wrong bootloader prevents the board from
being updated without a hardware debugger. Keeping the original bootloader
assures that in case of an error, the board can be fully reflashed. However,
keeping the original bootloader takes up 152 KB.

### Using the CLUEBOOT USB Drive

> **NOTE** Uploading the bootloader will not change any ability to upload software to the Clue nRF52840. The original bootloader
will not be overwrittn. All other software will work as expected.

Connect the Clue nRF52840 to the computer and double press the RESET button. A USB drive labeled `CLUEBOOT` should show up.

Drag and drop the [bootloader](https://github.com/tock/tock-bootloader/releases/download/clue_nrf52840-1.1.2/tock-bootloader.clue_nrf52840.1.1.2.uf2) to the `CLUEBOOT` drive and wait for a few seconds.

The board will reset and the bootloader should be running on it. The red LED should light up.

### Using the Adafruit nRF52 bootloader

To flash the tock-bootloader we use the Adafruit version of nRFUtil tool to communicate with the bootloader
on the board which then flashes the kernel. This requires that the bootloader be
active. To force the board into bootloader mode, press the button on the back of the board
twice in rapid succession. You should see the red LED pulse on and off.

Use the `make flash-bootloader` command to flash [Tock Bootloader](https://github.com/tock/tock-bootloader) to the board.

```bash
$ make flash-bootloader
```

If the flashing is sucessful, the LED on the back should turn on.

### Building the bootloader

This step is optional, as a prebuilt bootloader is provided as a
[tock-bootloader.clue_nrf52.v1.1.2.bin](https://github.com/tock/tock-bootloader/releases/download/clue_nrf52840-1.1.2/tock-bootloader.clue_nrf52840.1.1.2.bin).

To build the bootloader yourself, please follow the instructions in the Tock Bootloader's [documentation](https://github.com/tock/tock-bootloader/tree/master/boards) for CLUE nRF52840.


## Programming the Kernel


At this point you should be able to simply run `make program` in this directory
to install a fresh kernel.

## Programming Applications

After building an application, you can use `tockloader install` to install it.

For example:

```shell
$ cd libtock-c/examples/blink
$ make
$ tockloader install
```
## Using without tock-bootloader

Make the following changes to use Tock directly with the original bootloader:

```
# edit layout.ld
rom (rx)  : ORIGIN = 0x00026000, LENGTH = 360K
prog (rx) : ORIGIN = 0x00080000, LENGTH = 512K

# edit main.rs
fn baud_rate_reset_bootloader_enter() {
    unsafe {
        // 0x4e is the magic value the Adafruit nRF52 Bootloader expects
        // as defined by https://github.com/adafruit/Adafruit_nRF52_Bootloader/blob/master/src/main.c
        // NRF52_POWER.unwrap().set_gpregret(0x90);
        // uncomment to use with Adafruit nRF52 Bootloader
        NRF52_POWER.unwrap().set_gpregret(0x4e);
        cortexm4::scb::reset();
    }
}

# flash
make flash-kernel
```

## Kernel Panic

If the kernel panics, it might be difficult to enter the bootloader to replace the kernel. 
Use the following steps:

1. Enter the Adafruit nRF52 bootloader by double pressing RESET
2. Drag and drop another UF2 file that is larger than 70 KB to the `CLUEBOOT` drive. This will overwrite the tock-bootloader kernel that paniced.
3. Enter the Adafruit nRF52 bootloader by double pressing RESET
4. Drag and drop the tock bootloader UF2 file
5. Use tockloader to flash a new kernel
## Userspace Resource Mapping

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

### Sensors

The following sensors are working so far

| Sensor | Physical Element    |
|-------------------|---------------------|
| Proximity           | APDS9960              |
| Temperature & Humidity | SHT31 |


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
