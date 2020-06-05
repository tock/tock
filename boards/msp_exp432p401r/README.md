MSP432 Evaluation kit MSP432P401R MCU
========================================================

For more details [visit the Evaluation kit
website](https://www.ti.com/tool/MSP-EXP432P401R).

**Note:** This description was more or less copied from the stm32f3discovery board because the 
approach of flashing apps was also taken from there because there.

## Flashing the kernel

The kernel can be programmed using OpenOCD. `cd` into `boards/msp_exp432p401r`
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
    target/thumbv7em-none-eabi/debug/msp-exp432p401r.elf \
    target/thumbv7em-none-eabi/debug/msp-exp432p401r-app.elf
```

For example, you can update `Makefile` as follows (The Makefile contains these lines already,
they just have to be adapted to your own paths):

```
APP=../../../libtock-c/examples/c_hello/build/cortex-m4/cortex-m4.tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

.PHONY: program
program: target/$(TARGET)/debug/$(PLATFORM).elf
        arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
        $(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(KERNEL_WITH_APP); verify_image $(KERNEL_WITH_APP); reset; shutdown"
```

After setting `APP`, `KERNEL`, `KERNEL_WITH_APP`, and `program` target
dependency, you can do

```bash
$ make program
```

to flash the image.