Adafruit CLUE - nRF52840 Express with Bluetooth LE
==================================================

<img src="https://cdn-learn.adafruit.com/assets/assets/000/087/843/medium640/adafruit_products_Clue_top_angle.jpg?1580406577" width="35%">

The [Adafruit CLUE - nRF52840 Express with Bluetooth LE](https://www.adafruit.com/product/4500) is a
board based on the Nordic nRF52840 SoC. Includes the
following sensors:

- 9 axis inertial sensor
- humidity and temperature sensor
- barometric sensor
- microphone
- gesture, proximity, light color and light intensity sensor

It has thhe same form factor with BBC:Microbit


## Getting Started

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

You will need the bossac bootloader tool:

```shell
$ pip3 install --user adafruit-nrfutil
```

## Programming the Kernel

To program the kernel we use the BOSSA tool to communicate with the bootloader
on the board which then flashes the kernel. This requires that the bootloader be
active. To force the board into bootloader mode, press the button on the board
twice in rapid succession. You should see the yellow LED pulse on and off.

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

This is currently a weakness of the Nano 33 board as flashing applications is
not as ergonomic as Tock expects. Right now, you should be able to flash a
single application. For example, to flash the "blink" app, first compile it:

```
$ git clone https://github.com/tock/libtock-c
$ cd libtock-c/examples/blink
$ make
```

This previous step will create a TAB (`.tab` file) that normally tockloader
would use to program on the board. However, tockloader is currently not
supported. As a workaround, we can directly program a single app. To load the
blink app, first press the button on the board twice in rapid succession to
enter the bootloader, and then:

```
$ bossac -i -e -o 0x20000 -w build/cortex-m4/cortex-m4.tbf -R
```

That tells the BOSSA tool to flash the application in the Tock Binary Format to
the correct offset (the app will end up at address 0x30000). You may also need
to pass the `--port` flag.

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
output is output over the USB CDC connection and so should be visible as part
of the output of `tockloader listen`, however if your kernel panics so early
that the USB connection has not yet been established you will be unable to view
any panic output. In this case, you can modify the panic handler to instead
output panic information over the UART pins, but you will have to separately interface
with the UART pins on the board in order to observe the serial output.
