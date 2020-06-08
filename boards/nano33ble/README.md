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

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

You will need the bossac bootloader tool:

```shell
$ git clone https://github.com/arduino/BOSSA
$ cd BOSSA
$ make bossac
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

## UART Debugging

Currently the Nano 33 board file uses the UART peripheral for all UART
debugging. However, that UART is not connected to the USB header on the board.
So, all `debug!()` messages or application `printf()` calls will not be
displayed. If you want to see the debug output, you need to connect to the two
UART pins on the Nano 33 headers. You should be able to connect a serial <-> USB
converter, like an FTDI board, to retrieve the UART output.
