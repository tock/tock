BBC Micro:bit v2 - nRF52833 with Bluetooth LE
==================================================

<img src="https://cdn.sanity.io/images/ajwvhvgo/production/a7f49eb570ce06cf107dde7babaa5201411a41a1-660x720.jpg?q=80&fit=max&auto=format" width="35%">

The [BBC Micro:bit v2 - nRF52833 with Bluetooth LE](https://microbit.org/new-microbit/) is a
board based on the Nordic nRF52833 SoC. It includes the
following sensors:

- 5x5 LED Matrix
- LSM303AGR or FXOS8700CQ compass and accelerometer
- BLE
- speaker
- microphone

## Getting Started

First, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

## Flashing the kernel

The kernel can be programmed using OpenOCD. `cd` into `boards/microbit_v2`
directory and run:

```bash
$ make flash

(or)

$ make flash-debug
```

> **Note:** Unlike other Tock platforms, the default kernel image for this
> board will clear flashed apps when the kernel is loaded. This is to support
> the non-tockloader based app flash procedure below. To preserve loaded apps,
> comment out the `APP_HACK` variable in `src/main.rs`.

## Flashing app

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

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
    arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	$(OPENOCD) $(OPENOCD_OPTIONS) -c "program $(KERNEL_WITH_APP); verify_image $(KERNEL_WITH_APP); reset; shutdown;"
```

After setting `APP`, `KERNEL`, `KERNEL_WITH_APP`, and `program` target
dependency, you can do

```bash
$ make program
```

to flash the image.
