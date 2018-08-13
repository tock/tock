# TI LAUNCHXL-CC26x2/CC13x2 SimpleLink Wireless MCU LaunchPad Development Kit

The [launchpad launchxl CC26X2R1](http://www.ti.com/tool/LAUNCHXL-CC26X2R1) is a
platform based on the CC26x2 MCU by Texas Instrument, an SoC with an ARM
Cortex-M4 and a multi-functional Radio (BLE, IEEE, FM). The kit is i2c
compatible, and can be extended with several sensors using i2c.

The technical reference manual for the cc26x2 can be found
[here](http://www.ti.com/lit/ug/swcu185/swcu185.pdf).

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md).

There are two options for flashing and debugging the board:

  1. OpenOCD + on-board debugger: The launchxl boards have an on-board debug-chip (the XDS110) to
     allow flashing and debugging the chip over USB that works with OpenOCD.
     However, to use the on-board debugger, you need to upgrade its firmware,
     and use an unreleased version of OpenOCD that can interface with it.

  2. JLinkExe + external JTag debugger: You can connect an external debugger to
     the JTag port (labled "Target In") and use JLinkExe/JLinkGDBServer to
     flash and debug the chip. This is currently a more seemless software
     experience, but requires additinoal hardware (the debugger).

### Hardware Setup

The jumpers on the configurable headers change which debug features are
available.

In order to interact with the serial console over USB, you need to attach
jumpers to the `RX` and `TX` pins.

For flashing and debugging, you need different sets of jumpers for each strategy.

If you're using **OpenOCD**, ensure all the jumpers are attached to the
configurable header pins. Specifically, for flashing and debugging you need _at
least_:

  * `GND`
  * `3V3`
  * `RST`
  * `TMS`
  * `TCK`
  * `TDO`
  * `TDI`

If you're using **JLinkExe/JLinkGDBServer**, make sure to __*detach*__ the
`TMS`, `TCK`, `TDO`, and `TDI` pins. If you don't, JLinkExe will fail to
connect and print an error that it cannot find ICE-PICK (because TI is really
serious about useful error messages).

```
InitTarget: Can not find ICE-Pick (IDCODE mismatch).
```

### Update the XDS110 firmware

_Only necessary for OpenOCD_

The launchpad has TI's XDS110 JTAG device that can be used to program it.
However, OpenOCD requires a recent version of the firmware that may not be on
the board. To update, follow the steps
[here](http://processors.wiki.ti.com/index.php/XDS110#Updating_the_XDS110_Firmware).
Note: you only need to download CCS to do this, the `firmware.bin` file is in
the xds110 folder.

### Install OpenOCD from Git

_Only necessary for OpenOCD_

As of July 2018, the released version of OpenOCD (v0.10) does not include
support for the XDS110 on-board debugger. However, support is available from
source since May 2018 so you must install a version of OpenOCD from a recent
Git snapshot.

```bash
$ git clone https://git.code.sf.net/p/openocd/code openocd
```

Refer to the OpenOCD README for specific installation instructions, but it is just a normal autotools project, so the following should work:

```bash
$ cd openocd
$ ./bootstrap
$ ./configure --prefix $HOME/.local
$ make install
```

### Tockloader

Tockloader supports flashing the launchpad using OpenOCD.

```bash
$ tockloader <command> --board launchxl-cc26x2r1 --openocd
```

### Customer Configuration (CCFG)

__*IMPORTANT*: This step is both necessary (at least once) and a bit dangerous.
Proceed carefully.__

The CC26x2/CC13x2 series of chips contains a set of customer configuration
registers (CCFG) that modify chip behavior in various ways, including the radio
MAC address, voltage, and startup behavior. The registers live at a very high
address, but can be written using the last page in flash (offset 0x56000).

By default, the chip is configured to start a hardware bootloader that loads a
payload binary over the UART. _We do not use this bootloader_. Before Tock will
properly boot, you have to change the CCFG registers.

A reasonable set of CCFG values is in `src/ccfg.rs`, which compiles to a
separate target binary and can be flashed directly to offset of the first CCFG
register (0x57FA8).

If you're using OpenOCD:

```bash
$ make flash-ccfg
```

If you're using JLinkExe

```bash
$ make flash-ccfg-jlink
```

### Flashing the kernel

The Makefile targets `flash` (for OpenOCD) and `flash-jlink`) (for JLinkExe) build and load the kernel.

OpenOCD:

```bash
$ make flash       # make and flash the kernel
```

JLinkExe:

```bash
$ make flash-jlink       # make and flash the kernel
```

### Flashing processes

You can flash processes using Tockloader.

OpenOCD:

```bash
$ tockloader install --board launchxl-cc26x2r1 --openocd [PATH_TO_PROCESS]
```

JLinkExe:

```bash
$ tockloader install --board launchxl-cc26x2r1 --jlink [PATH_TO_PROCESS]
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

When the board panics or crashes, the RED led will blink rapidly.
