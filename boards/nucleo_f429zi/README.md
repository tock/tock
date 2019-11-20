STM32 Nucleo-64 development board with STM32F429ZI MCU
======================================================

For more details [visit NUCLEO-F429ZI
website](https://www.st.com/en/evaluation-tools/nucleo-f429zi.html).

## Flashing the kernel

The kernel can be programmed using OpenOCD. `cd` into `boards/nucleo_f429zi`
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
    target/thumbv7em-none-eabi/debug/nucleo_f429zi.elf \
    target/thumbv7em-none-eabi/debug/nucleo_f429zi-app.elf
```

For example, you can update `Makefile` as follows.

```
APP=../../../libtock-c/examples/c_hello/build/cortex-m4/cortex-m4.tbf
KERNEL=$(CWD)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(CWD)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

.PHONY: program
program: target/$(TARGET)/debug/$(PLATFORM).elf
        arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
        sudo $(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(KERNEL_WITH_APP); verify_image $(KERNEL_WITH_APP); reset; shutdown"
```

After setting `APP`, `KERNEL`, `KERNEL_WITH_APP`, and `program` target
dependency, you can do

```bash
$ make program
```

to flash the image.
