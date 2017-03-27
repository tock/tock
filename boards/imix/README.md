imix: Platform-Specific Instructions
=====================================

Kernel and userland software can be flashed onto the imix using
[openocd](http://openocd.org/). We require at least version `0.8.0`.

```bash
(Linux): sudo apt-get install openocd
(MacOS): brew install open-ocd
```


## Flashing the kernel

To program the Tock kernel onto the imix, `cd` into the `boards/imix` directory
and run:

```bash
$ make flash
```

This will build `boards/imix/target/sam4l/release/imix/imix` and use openocd to
flash it to the board.


## Flashing apps

All user-level code lives in the `userland` subdirectory. This includes a
specially compiled version of newlib, a user-level library for talking to the
kernel and specific drivers and a variety of example applications.

To compile an app, `cd` to the desired app and `make`. For example:

```bash
$ cd userland/examples/blink/
$ make TOCK_BOARD=imix
```

This will build the app and generate a binary in Tock Binary Format (using the
`elf2tbf` utility): `userland/examples/blink/build/cortex-m4/app.bin`. This
binary should be flashed separately from the kernel.

Apps can be built and automatically uploaded from the root directory of Tock:

```bash
$ make TOCK_BOARD=imix examples/blink
```

Like the kernel, apps can be uploaded with `make flash`:

```bash
$ cd userland/examples/blink/
$ make TOCK_BOARD=imix flash
```

This builds and loads only a single app. Tock is capable of running multiple apps
concurrently. **TODO**

## Debugging

To debug a loaded kernel with `openocd`:

```bash
$ cd boards/imix/
$ openocd -f connect.cfg
```

Then, in another terminal (assuming you have loaded a kernel image built using
the `release` profile):

```bash
$ cd boards/imix/
$ arm-none-eabi-gdb target/sam4l/release/imix.elf
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

Console interaction may not work well for the imix board at present, but here
are a few notes that may be helpful.

It may be possible to connect to the FTDI chip by plugging a USB cable into the
DBG\_USB port (the one closer to the middle), and then use `miniterm.py` to
open that serial port:

```bash
$ miniterm.py --dtr 0 --rts 1 /dev/ttyUSB0
```

Miniterm is similar to `screen` but lets you control the DTR and RTS lines,
which we re-purpose to control the sam4l reset line.  Note that `--rts 1`
shouldn't have any impact now, but may mitigate power-supply problems with
console interaction. On most simple applications it won't make a difference.

You can install the `miniterm` script from the "pySerial" pip package:

```bash
$ pip install pyserial --user
```

