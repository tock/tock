STM32 Discovery kit development board with STM32F303 MCU
========================================================

For more details [visit the Discovery kit
website](https://www.st.com/en/evaluation-tools/stm32f3discovery.html).

## Flashing the kernel

The kernel can be programmed using OpenOCD. `cd` into `boards/stm32f3discovery`
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

Apps are built out-of-tree. Once an app is built, you can add the path to it in the Makefile (APP variable), then run:
```bash
$ make program
```

or you can define the APP variable in the command line

```bash
$ make program APP=path_to_tbf
```

