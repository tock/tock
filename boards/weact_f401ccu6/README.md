WeAct STM32F401CCU6 Core Development Board
======================================================

For more details [visit WeActTC/MiniF4-STM32F4x1
Github repo](https://github.com/WeActTC/MiniF4-STM32F4x1).

## Flashing the kernel

The kernel can be programmed using OpenOCD. `cd` into `boards/weact_f401ccu6`
directory and run:

```bash
$ make flash

(or)

$ make flash-debug
```

Expects ST-LINK V2-1, if using V2 you can change lines 4-5 in
`openocd.cfg` to:
```bash
hla_device_desc "ST-LINK/V2"
hla_vid_pid 0x0483 0x3748
```

> **Note:** Unlike other Tock platforms, the default kernel image for this
> board will clear flashed apps when the kernel is loaded. This is to support
> the non-tockloader based app flash procedure below.

## Flashing an app

Apps are built out-of-tree. Once an app is built
(see [libtock-c](https://github.com/tock/libtock-c)), you can use
`arm-none-eabi-objcopy` with `--update-section` to create an ELF image with the
apps included.

```bash
$ arm-none-eabi-objcopy  \
    --update-section .apps=../../../libtock-c/examples/c_hello/build/cortex-m4/cortex-m4.tbf \
    ../../target/thumbv7em-none-eabihf/release/weact-f401ccu6.elf \
    ../../target/thumbv7em-none-eabihf/release/weact-f401ccu6-app.elf
```

For example, you can update `Makefile` as follows.

```
APP=../../../libtock-c/examples/c_hello/build/cortex-m4/cortex-m4.tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
        arm-none-eabi-objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
        $(OPENOCD) $(OPENOCD_OPTIONS) -c "init; reset halt; flash write_image erase $(KERNEL_WITH_APP); verify_image $(KERNEL_WITH_APP); reset; shutdown"
```

After setting `APP`, `KERNEL`, `KERNEL_WITH_APP`, and `program` target
dependency, you can do

```bash
$ make program
```

to flash the image.


**NOTE:** [Tockloader](https://github.com/tock/tockloader) support may be available in the future and may be used to upload apps.
