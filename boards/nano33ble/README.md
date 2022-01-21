Arduino Nano 33 BLE
===================

<img src="https://store-cdn.arduino.cc/usa/catalog/product/cache/1/image/1040x660/604a3538c15e081937dbfbd20aa60aad/a/b/abx00031_featured.jpg" width="35%">

The [Arduino Nano 33 BLE](https://store.arduino.cc/usa/nano-33-ble) and [Arduino
Nano 33 BLE Sense](https://store.arduino.cc/usa/nano-33-ble-sense) are compact
boards based on the Nordic nRF52840 SoC. The "Sense" version includes the
following sensors:

- 9 axis inertial sensor
- humidity and temperature sensor
- barometric sensor
- microphone
- gesture, proximity, light color and light intensity sensor


## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md).

The Nano 33 comes pre-installed with a
[version](https://github.com/arduino/ArduinoCore-nRF528x-mbedos/tree/master/bootloaders/nano33ble)
of the BOSSA bootloader that Arduino uses on various boards. Unfortunately this
bootloader is not well suited for Tock development. Specifically, it doesn't
support reading from the board, so there is no way to automatically determine
what type of board it is or what is already installed. It's also [not open
source](https://github.com/arduino/ArduinoCore-nRF528x-mbedos/issues/23) (at
least as of December 2020).

For Tock development we need to replace the bootloader with the [Tock
Bootloader](https://github.com/tock/tock-bootloader). The Tock bootloader
allows a lot more flexibility with reading and writing the board, and is also
implemented on top of Tock itself.

This guide will walk through how to install the Tock bootloader, and describe
what is happening along the way. Our goal is to get the Tock bootloader flashed
to the nRF52840 at address 0x0 (overwriting the bossa bootloader). The bossa
bootloader does not have a mechanism for updating itself, however, so we have to
do this in a bit of a roundabout manner.

We also have a guide for restoring the BOSSA bootloader in case you want to go
back.

1. The first step is you will need the bossac tool. This tool is required to use
   the existing bootloader the board ships with.

    You can compile this tool from source:

	```shell
	$ git clone https://github.com/arduino/BOSSA
	$ cd BOSSA
	$ make bossac
	```

	Then you will need to add `BOSSA/bin` to your `$PATH` variable so that your
	system can find the `bossac` program.

2. Next we will use the bossa bootloader to load a copy of the Tock bootloader.
   The bossa bootloader expects that all application code (i.e. not the
   bootloader) starts at address 0x10000. That is, when the bootloader finishes
   it starts executing at address 0x10000.

    So, we will load a copy of the Tock bootloader to address 0x10000. That also
    means we need a version of the bootloader compiled to run at address
    0x10000. This bootloader has already been compiled for you.

    To load the first Tock bootloader ensure the Nano 33 is in bootloader mode
    by double pressing the reset button (the light should pulse), and then:

    ```shell
    $ bossac -e -w bootloaders/tock-bootloader.nano33ble.v1.1.0.0x10000.bin
    ```

    Now the board should boot into the Tock bootloader. When the Tock bootloader
    is active it will turn on the "ON" LED. It will not pulse the yellow LED.

    You can test that this step worked by using tockloader. A simple test is to
    run:

    ```
    $ tockloader info
    ```

    You should see various properties of the board. This indicates that tockloader
    is able to communicate with the temporary bootloader.

3. Now we use the temporary Tock bootloader to flash the real one
   at address 0x0. When the board boots it should run the Tock bootloader
   flashed at address 0x10000. To then flash the correct bootloader, run:

    ```shell
    $ tockloader flash bootloaders/tock-bootloader.nano33ble.v1.1.0.0x00000.bin --address 0
    ```

4. At this point we have two copies of the Tock bootloader installed: the intended
   one at address 0x10000 and the temporary one at address 0x0. By default, the first bootloader
   will jump to the second, and the second will continue running if no kernel is installed.
   To reduce confusion, we can remove the second bootloader. To do this, we will overwrite
   the start of the bootloader with zeros.

   First, double tap the reset button. This will cause the first bootloader to stay
   active. You should see the "ON" LED on.

   Then, overwrite the second bootloader:

    ```shell
    $ tockloader write 0x10000 512 0
    ```

5. That's it! You now have the Tock bootloader. All `tockloader` commands should
   now work.

    You can test Tockloader by running:

    ```shell
    $ tockloader info
    ```

    You should see various properties of the board displayed.



## Programming the Kernel

You should be able to simply run `make program` in this directory
to install a fresh kernel.

```
$ make program
```

## Programming Applications

After building an application, you can use `tockloader install` to install it.

For example:

```shell
$ cd libtock-c/examples/blink
$ make
$ tockloader install
```

### Userspace Resource Mapping

This table shows the mappings between resources available in userspace
and the physical elements on the Nano 33 BLE board.

| Software Resource | Physical Element    |
|-------------------|---------------------|
| GPIO[2]           | Pin D2              |
| GPIO[3]           | Pin D3              |
| GPIO[4]           | Pin D4              |
| GPIO[5]           | Pin D5              |
| GPIO[6]           | Pin D6              |
| GPIO[7]           | Pin D7              |
| GPIO[8]           | Pin D8              |
| GPIO[9]           | Pin D9              |
| GPIO[10]          | Pin D10             |
| LED[0]            | Tri-color LED Red   |
| LED[1]            | Tri-color LED Green |
| LED[2]            | Tri-color LED Blue  |

## Debugging

The Nano 33 board uses a virtual serial console over USB to send debugging info
from the kernel and print messages from applications. You can use whatever your
favorite serial terminal program is to view the output. Tockloader also
supports reading and writing to a serial console with `tockloader listen`.

### Kernel Panics

If the kernel or an app encounter a `panic!()`, the panic handler specified in
`io.rs` is called. This causes the kernel to stop. You will notice the yellow
LED starts blinking in a repeating but slightly irregular pattern. There is also
a panic print out that provides a fair bit of debugging information. That panic
output is output over the USB CDC connection and so should be visible as part of
the output of `tockloader listen`, however if your kernel panics so early that
the USB connection has not yet been established you will be unable to view any
panic output. In this case, you can modify the panic handler to instead output
panic information over the UART pins, but you will have to separately interface
with the UART pins on the board in order to observe the serial output.

## Factory Reset

To restore the BOSSA bootloader we can largely reverse the steps used to install
the Tock bootloader.

1. First we need to install a temporary copy of the Tock bootloader.

    ```shell
    $ tockloader flash bootloaders/tock-bootloader.nano33ble.v1.1.0.0x10000.bin --address 0x10000
    ```

2. Now we can restore the BOSSA bootloader.

    ```shell
    $ wget https://github.com/arduino/ArduinoCore-nRF528x-mbedos/raw/00ce64c29c4c2e139335b930fe693a00363936aa/bootloaders/nano33ble/bootloader.bin
    $ tockloader flash bootloader.bin --address 0x0
    ```

    Double clicking reset should enter the bossa bootloader now.

### Using JTAG to flash the Nano 33 BLE

When flashing bootloaders there is some risk of corrupting the bootloader such
that tools like Tockloader and BOSSA can no longer program the board. We try to
prevent this, but it can happen. To restore the bootloader you must use the JTAG
connection and an external JTAG programmer to flash the nRF52840 with a
bootloader.

One method for doing this requires a nRF52840dk (the PCA10056 board from
Nordic). This development board includes on-board JTAG hardware that can flash
an external chip. You will need:

- The
  [nRF52840dk](https://www.nordicsemi.com/Software-and-Tools/Development-Kits/nRF52840-DK).
- Three 0.1" pitch square-head male-to-female jumper wires ([example from
  Pololu](https://www.pololu.com/category/67/male-female-premium-jumper-wires)).
- Three 0.1" pitch square-head jumper wires. Can have any ends.
- One 0.1" jumper or female-female jumper wire.

On the bottom of the Nano 33 BLE are five circular pads underneath the BLE radio.
When looking at the BOTTOM, they are:

```
3V3   --> •   • <-- RESET

SWCLK --> •

GND   --> •   • <-- SWDIO
```

The nRF52840dk has a header P20 which can be used for JTAG programming an
external chip. Here are the connections I made:

1. Jumper P20 pin 2 (VDD nRF) to P20 pin 3 (VTG). This enables external
   programming.
2. Connect a M/F jumper to P20 pin 4 (SWDIO).
3. Connect a M/F jumper to P20 pin 5 (SDDCLK).
4. Connect a M/F jumper to P20 pin 7 (RESET).
5. Plug a USB cable into the Nano 33 BLE and connect it to your computer. This
   will power the Nano 33 BLE and provide ground.

Now take the male end of the three jumpers you just connected, along with three
other 0.1" jumpers, and form them in a 2x3 bundle that matches the pad mapping
on the back of the Nano 33 BLE. I used tape to hold the 2x3 bundle together. You
will use the male pins and press them onto the pads to connect to the Nano 33
BLE. The three other jumpers are just for spacing to make it easier to connect
the wires which matter. These jumpers do not actually have to make contact with
the Nano 33 BLE.

Make sure that SW6 on the nRF52840dk is on "DEFAULT", and SW8 is on "ON".

Now to flash a bootloader on to the Nano 33 BLE:

1. Prepare your terminal. Go to `/boards/nano33ble` and enter (but do not run)
   `tockloader flash --address 0
   bootloaders/tock-bootloader.nano33ble.v1.1.0.0x00000.bin --board nrf52dk
   --jlink --debug`.
2. Ensure the LED5 on the nRF52840dk is on (it might flash a bit, that is OK).
3. Hold the jumper bundle against the JTAG pads on Nano 33 BLE. This can be a
   bit tricky, but your goal is to get all the pins to make contact with the
   pads.
4. Press [enter] on your terminal to run the command. You want to see output
   like `Verify successful.` and other messages suggesting the nRF52840dk was
   able to flash the chip. If you see `Connecting to target via SWD Cannot
   connect to target.` that means it did NOT work. You will need to keep
   adjusting the pins to make sure they make contact with the Nano 33 BLE. Also,
   make sure LED5 on the nRF52840dk remains on. This LED might go off if things
   get connected weird. To reset things switch SW8 to OFF and then back to ON.

Assuming everything goes correctly you should have restored the bootloader! You
can now use tockloader as normal.

#### Related resources

I used a couple pages to help figure all of this out in January 2021:

- https://devzone.nordicsemi.com/f/nordic-q-a/55203/programming-external-board-bc832-with-nrf52840-dk
- https://devzone.nordicsemi.com/f/nordic-q-a/20536/programming-external-custom-nrf52832-board-using-nrf52-dk
- https://devzone.nordicsemi.com/f/nordic-q-a/32661/external-programming-with-nrf5-dk-p20
