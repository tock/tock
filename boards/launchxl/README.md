Platform Specific Instructions: LAUNCHXL-CC26X2R1
=================================================

The [launchpad launchxl CC26X2R1](http://www.ti.com/tool/LAUNCHXL-CC26X2R1) is a
platform based on the CC26x2 MCU by Texas Instrument, an SoC with an ARM
Cortex-M4 and a multi-functional Radio (BLE, IEEE, FM). The kit is i2c
compatible, and can be extended with several sensors using i2c.

The technical reference manual for the cc26x2 can be found
[here](http://www.ti.com/lit/ug/swcu185/swcu185.pdf), and it shares many
properties with other MCUs in the same family (cc26xx). Most of the implemented
features of the cc26xx family has been done by using the [reference manual for
cc26x0](http://www.ti.com/lit/ug/swcu117h/swcu117h.pdf), which is more
comprehensible than the manual for cc26x2.

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md).

### Update the XDS110 firmware

The launchpad has TI's XDS110 JTAG device that can be used to program it.
However, OpenOCD requires a recent version of the firmware that may not be on
the board. To update, follow the steps
[here](http://processors.wiki.ti.com/index.php/XDS110#Updating_the_XDS110_Firmware).
Note: you only need to download CCS to do this, the `firmware.bin` file is in
the xds110 folder.

### Tockloader

Tockloader supports flashing the launchpad using OpenOCD. To do this, you need a
(recent) version of OpenOCD that contains [this
file](https://github.com/ntfreak/openocd/blob/master/tcl/board/ti_cc26x2_launchpad.cfg).

Also, you need to ensure the jumpers are set correctly. You must have jumpers
on the `GND`, `3V3`, `RESET`, `TMS`, `TCK`, `TDO`, and `TDI` pins.

Then, using Tockloader should work like so:

```bash
$ tockloader <command> --board launchxl-cc26x2r1 --openocd
```

### Flashing

You are able to use the Makefile target `flash` to load the kernel onto the
launchxl board.

```bash
$ make flash       # make and flash the kernel
```

### Flashing apps

You can flash apps by navigating to their directory, and by invoking `make
flash` once again, or call Tockloader directly:

```bash
$ tockloader install --board launchxl-cc26x2r1 --openocd
```

### Debugging

You need to use openocd together with gdb in order to debug the launchxl board
using JTAG. However, you'll need to build OpenOCD with extra applied patches
until the next version has been released.

Clone the repository and apply the patches:

```bash
$> git clone https://git.code.sf.net/p/openocd/code openocd
$> cd openocd
$> git pull http://openocd.zylin.com/openocd refs/changes/22/4322/2
$> git pull http://openocd.zylin.com/openocd refs/changes/58/4358/1
```

Once cloned and the patches has been applied, read the readme in order to build
and install openocd.

Once flashed, launch openocd with the configuration specified at jtag/openocd.cfg:

```bash
$> openocd -f jtag/openocd.cfg
```

And then launch gdb

```bash
$> arm-none-eabi-gdb -x jtag/gdbinit
```

and it will automatically connect to the board.

**NOTE**: There is currently a problem using the `cortex_m SYSRESETREQ` command
in openocd in order to reset the board. This is also the default way we want to
reset..


### Panic/Crash

When the board panics or crashes, the RED led will be blinking frequently.
