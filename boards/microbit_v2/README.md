BBC Micro:bit v2 - nRF52833 with Bluetooth LE
==================================================

<img src="https://cdn.sanity.io/images/ajwvhvgo/production/a7f49eb570ce06cf107dde7babaa5201411a41a1-660x720.jpg?q=80&fit=max&auto=format" width="35%">

The [BBC Micro:bit v2 - nRF52833 with Bluetooth LE](https://microbit.org/new-microbit/) is a
board based on the Nordic nRF52833 SoC. It includes the
following sensors:

- 5x5 LED Matrix
- LSM303AGR compass and accelerometer
- BLE
- speaker
- microphone

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md)

## Bootloader

Tock uses [Tock Bootloader](https://github.com/tock/tock-bootloader) to program devices.

As MicroBit v2 has an on board debugger that provides several ways of programming it, is shipped without an actual bootloader.

There are two ways for flashing the bootloader:
 1. Using the MicroBit USB drive
 2. Using openocd

### Building the bootloader

This step is optional, as a prebuilt bootloader is provided as a [tock-bootloader.microbit_v2.vv1.1.1.bin](https://github.com/tock/tock-bootloader/releases/download/microbit_v2-vv1.1.1/tock-bootloader.microbit_v2.vv1.1.1.bin).

To build the bootloader yourself, please follow the instructions in the Tock Bootloader's [documentation](https://github.com/tock/tock-bootloader/tree/master/boards/microbit_v2-bootloader) for Micro:bit v2.
### Using the MicroBit USB Drive

> **NOTE** Uploading the bootloader will not change any ability to upload software to the MicroBit. The microbit board has another bootloader in the debug chip that provides normal software upload capabilites and that will not be overwritten. All other software will work as expected.

Connect then MicroBit to the computer. A USB drive labeled `MICROBIT` should show up.

Drag and drop the [tock-bootloader.microbit_v2.vv1.1.1.bin](https://github.com/tock/tock-bootloader/releases/download/microbit_v2-vv1.1.1/tock-bootloader.microbit_v2.vv1.1.1.bin) to the `MICROBIT` drive and wait for a few seconds.

The board will reset and the bootloader should be running on it. To check whether it's working, press and hold the Button A while pressing the reset button. The Microphone LED should light up.
### Using openocd
Use the `make flash-bootloader` command to flash [Tock Bootloader](https://github.com/tock/tock-bootloader) to the board.

```bash
$ make flash-bootloader
```

## Uploading the kernal

Make sure you have flashed [Tock Bootloader](https://github.com/tock/tock-bootloader) to the board.

Make sure you have [Tockloader](https://github.com/tock/tockloader) installed.

To upload the kernel, you must first enter in bootloader mode. Press and hold Button A while pressing the Reset button on the back of the board.
The board will reset and enter bootloader mode. This is signaled by turning on the Microphone LED.

In bootloader mode, run the `make program` command.

```bash
$ make program
```

Programming the kernal might take some time.

## Manage applications

Make sure you have flashed [Tock Bootloader](https://github.com/tock/tock-bootloader) to the board.

Make sure you have [Tockloader](https://github.com/tock/tockloader) installed.

To manage applications, please read the [Tockloader documentation](https://github.com/tock/tockloader/blob/master/docs/index.md).

> **_NOTE:_**  If you are using an older version of Tockloader, add `--page-size 512` at the end of the command line.
>
> ```bash
> $ tockloader ... --page-size 512
> ```


## Flashing without bootloader

### Memory layout

The kernel memory layout is different if there is no bootloader. Change the `layout.ld` file to:

```
MEMORY
{
  # with bootloader
  # rom (rx)  : ORIGIN = 0x00008000, LENGTH = 192K
  # without bootloader
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 256K
  prog (rx) : ORIGIN = 0x00040000, LENGTH = 256K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 128K
}
```

Not using a bootloader has the advantage of having an extra 64 KB of flash.
### Flashing the kernel

The kernel can be programmed using OpenOCD. `cd` into `boards/microbit_v2`
directory and run:

```bash
$ make flash

(or)

$ make flash-debug
```

### Flashing app

Please refer to the [tockloader](https://github.com/tock/tockloader) documentation to flash apps.

With bootloader
```bash
$ tockloader install app.tab
```

Without bootloader
```bash
$ tockloader --openocd --board microbit_v2 --bundle-apps install app.tab
```

> `--bundle-apps` seems to be needed due to an [openocd issue](https://github.com/tock/tockloader/issues/67)

## Troubleshooting

### Could not find MEM-AP to control the core

OpenOCD displays `Could not find MEM-AP to control the core` error when trying to connect to Micro:bit. This means that the Micro:bit has [control access port](https://infocenter.nordicsemi.com/topic/com.nordic.infocenter.nrf52832.ps.v1.1/dif.html?cp=2_2_0_15_1#concept_udr_mns_1s) protection enabled. 

Using openOCD, you can check if access protection is enabled by executing the command: 

```bash
$ openocd -f openocd.cfg -c "dap apreg 1 0x0c"
``` 

This command reads the register at address 0x0c in the access port at index 1 (the control access port's index). If it returns 0x0 then [access port protection is enabled](https://infocenter.nordicsemi.com/topic/com.nordic.infocenter.nrf52832.ps.v1.1/dif.html?cp=2_2_0_15_1_0_3#register.APPROTECTSTATUS).

#### Solution

Unlock the chip by executing the command:

```bash
$ openocd -f openocd.cfg -c "dap apreg 1 0x04 0x01"
```

Reset the device. 

> Note that this will erase all Flash and RAM as this is the only way to disable CTRL-AP protection.

> Another solution would be to use Nordic's [nrfjprog](https://infocenter.nordicsemi.com/topic/com.nordic.infocenter.tools/dita/tools/nrf5x_command_line_tools/nrf5x_command_line_tools_lpage.html) to unlock your chip via `$ nrfjprog -f NRF52 --recover`, but this requires the use of a JLink debugger (on-board nRF52 DKs or external).
