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

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

## Bootloader

Tock uses [Tock Bootloader](https://github.com/tock/tock-bootloader) to program devices.

As MicroBit v2 has an on board debugger that provides several ways of programming it, is shipped without an actual bootloader.

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
  # rom (rx)  : ORIGIN = 0x00010000, LENGTH = 192K
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

Apps are built out-of-tree. Once an app is built, you can use
`arm-none-eabi-objcopy` with `--update-section` to create an ELF image with the
apps included.

```bash
$ arm-none-eabi-objcopy  \
    --update-section .apps=../../../libtock-c/examples/c_hello/build/cortex-m4/cortex-m4.tbf \
    target/thumbv7em-none-eabi/debug/stm32f3discovery.elf \
    target/thumbv7em-none-eabi/debug/stm32f3discovery-app.elf
```

For example, you can update `Makefile` as follows.

```
APP=../../../libtock-c/examples/c_hello/build/cortex-m4/cortex-m4.tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

.PHONY: flash-app
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
    arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $(KERNEL_WITH_APP); verify_image $(KERNEL_WITH_APP); reset; shutdown;"
```

After setting `APP`, `KERNEL`, `KERNEL_WITH_APP`, and `program` target
dependency, you can do

```bash
$ make flash-app
```

to flash the image.
