Platform-Specific Instructions: EK-TM4C1294XL
=====================================

The [EK-TM4C1294XL Development Board](http://www.ti.com/tool/EK-TM4C1294XL) is a platform based around the TM4C1294NCPDT, a microcontroller with an ARM Cortex-M4F.

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md).

JTAG is the preferred method to program. The development board has an integrated JTAG debugger, you simply need to [install OpenOCD](../../doc/Getting_Started.md#optional-requirements).

## Flashing the kernel

To program the Tock kernel onto the EK-TM4C1294XL, `cd` into the `boards/ek-tm4c1294xl` directory
and run:

```bash
$ make flash
```
to flash a fresh kernel.

## Flashing apps

You can program an application via JTAG:
```bash
$ cd userland/examples/<app>
$ make TOCK_BOARD=ek-tm4c1294xl flash
```

