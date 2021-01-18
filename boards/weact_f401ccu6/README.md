WeAct STM32F401CCU6 Core Development Board
======================================================

For more details [visit WeActTC/MiniF4-STM32F4x1
Github repo](https://github.com/WeActTC/MiniF4-STM32F4x1).

## Flashing the kernel

The kernel can be programmed using OpenOCD. `cd` into `boards/weact_f401ccu6`
directory and run:

```bash
$ make flash
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
(see [libtock-c](https://github.com/tock/libtock-c)) and a tbf file is generated,
you can use `arm-none-eabi-objcopy` with `--update-section` to create an
ELF image with the apps included.

```bash
$ arm-none-eabi-objcopy  \
    --update-section .apps=../../../libtock-c/examples/blink/build/cortex-m4/cortex-m4.tbf \
    ../../target/thumbv7em-none-eabihf/release/weact-f401ccu6.elf \
    ../../target/thumbv7em-none-eabihf/release/weact-f401ccu6-app.elf
```

The board `Makefile` can also handle this process and upload a given app automatically.

```bash
$ make flash-app APP=<...>
```


**NOTE:** [Tockloader](https://github.com/tock/tockloader) support may be available in the future and may be used to upload apps.
