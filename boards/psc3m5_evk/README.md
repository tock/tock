PSOC™ Control C3M5 Evaluation Kit
===========================

<img src="https://assets.infineon.com/is/image/infineon/kit-psc3m5-evk-main-picture-kit-psc3m5-evk.png" width="40%">

The [PSOC™ Control C3M5 Evaluation Kit](https://www.infineon.com/evaluation-board/kit-psc3m5-evk) is a evaluation board for the PSOC Control C3M5 microcontroller, which is based on the Arm Cortex-M33 architecture.

## Getting started

Install `probe-rs`.\
OR\
OpenOCD from [ModusToolbox™ Programming Tools](https://softwaretools.infineon.com/tools/com.ifx.tb.tool.modustoolboxprogtools)

## Flashing the kernel

The kernel can be programmed by going inside the board's directory and running:
```bash
$ make flash # program for OpenOCD
```

## Flashing an app

Apps are built out-of-tree. Once an app is built, you must add the path to the generated TBF in the Makefile (APP variable), then run:
```bash
$ make flash APP=path/to/app.tbf # program for OpenOCD
```

This will generate a new ELF file that can be deployed on the board via gdb and probe-rs.
