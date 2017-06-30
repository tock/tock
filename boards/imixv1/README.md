imix v1: Platform-Specific Instructions
=====================================

This is the legacy board file for older versions of imix (version 1). It will
be removed in a future release of Tock.

Kernel and userland software can be flashed onto the imix using
[openocd](http://openocd.org/). We require at least version `0.8.0`.

```bash
(Linux): sudo apt-get install openocd
(MacOS): brew install open-ocd
```


## Flashing the kernel

To program the Tock kernel onto the imix, `cd` into the `boards/imixv1` directory
and run:

```bash
$ make flash
```

This will build `boards/imixv1/target/sam4l/release/imixv1/imixv1` and use openocd to
flash it to the board.


## Flashing apps

All user-level code lives in the `userland` subdirectory. This includes a
specially compiled version of newlib, a user-level library for talking to the
kernel and specific drivers and a variety of example applications.

To compile an app, `cd` to the desired app and `make`. For example:

```bash
$ cd userland/examples/blink/
$ make TOCK_BOARD=imixv1
```

This will build the app and generate a binary in Tock Binary Format (using the
`elf2tbf` utility): `userland/examples/blink/build/cortex-m4/app.bin`. This
binary should be flashed separately from the kernel.

Apps can be built and automatically uploaded from the root directory of Tock:

```bash
$ make TOCK_BOARD=imixv1 examples/blink
```

Like the kernel, apps can be uploaded with `make flash`:

```bash
$ cd userland/examples/blink/
$ make TOCK_BOARD=imixv1 flash
```

This builds and loads only a single app. Tock is capable of running multiple apps
concurrently. **TODO**

## Debugging

To debug a loaded kernel with `openocd`:

```bash
$ cd boards/imixv1/
$ openocd -f connect.cfg
```

Then, in another terminal (assuming you have loaded a kernel image built using
the `release` profile):

```bash
$ cd boards/imixv1/
$ arm-none-eabi-gdb target/sam4l/release/imixv1.elf
(gdb) target remote localhost:3333
(gdb) monitor reset halt
(gdb) break <?>   # try tab-completion to find useful name-mangled breakpoints
(gdb) continue
```

You may issue other commands to `openocd` by prefixing them with `monitor`, as
above.  The manual for that utility is likely available on your system via
`info openocd`; an HTML version should be available on
[the website](http://openocd.org/).  You may also issue commands directly to a
running instance of `openocd` via telnet:

```bash
telnet localhost 4444
```

## Console I/O

Connect to the FTDI chip by plugging a USB cable into the DBG\_USB port (the
one closer to the middle), and then use `miniterm.py` to open that serial port:

```bash
$ miniterm.py --dtr 0 --rts 1 /dev/ttyUSB0 115200
```

(Note that you may need to configure your system to allow user access to the
USB serial port device.)

Miniterm is a terminal emulator that allows control over the DTR and RTS lines,
which the imix board re-purposes to control the SAM4L's reset line.  You may
type `CTRL-T`, `CTRL-D` to toggle DTR and thus reset the chip; doing this a
second time will then restart it.

You can install the `miniterm` script from the `pySerial` pip package:

```bash
$ pip install pyserial --user
```

