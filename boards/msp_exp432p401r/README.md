MSP432 Evaluation kit MSP432P401R MCU
=====================================

For more details [visit the Evaluation kit website](https://www.ti.com/tool/MSP-EXP432P401R).

## Flashing the kernel

The kernel can be programmed using OpenOCD. `cd` into `boards/msp_exp432p401r`
directory and run:

```bash
$ make flash

(or)

$ make flash-debug
```

**Note:** Make sure to use the latest git-openOCD-version as there is currently no support for the
XDS110 debug-probe in the pre-built binaries!

## Flashing an app

Apps are also flashed via openOCD. Make sure, your app is was compiled and converted into the TBF
(Tock Binary Format) format. Then `cd` into the `boards/msp_exp432p401r`directory and run:

```bash
$ make flash-app APP=<path_to_tbf_file>

(e.g.)

$ make flash-app APP=../../../libtock-c/examples/c_hello/build/cortex-m4/cortex-m4.tbf
```
