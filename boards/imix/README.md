imix: Platform-Specific Instructions
=====================================

This board file is for imix version 2.


## Flashing the kernel

To program the Tock kernel onto the imix, `cd` into the `boards/imix` directory
and run:

```bash
$ make program
```

This will build `boards/imix/target/sam4l/release/imix/imix` and use tockloader to
flash it to the board.

If you have connected to the board over a JTAG interface, you should instead
flash the kernel with:

```bash
$ make flash
```

## Flashing apps

To compile an app, `cd` to the desired app and `make`. For example:

```bash
$ git clone https://github.com/tock/libtock-c.git
$ cd libtock-c/examples/blink/
$ make TOCK_BOARD=imix
```

This will build the app and generate a binary in Tock Binary Format and create a
TAB (Tock Application Bundle) using the `elf2tab` utility:
`blink/build/blink.tab`.

Apps can be uploaded with `make program` (to use the serial bootloader), but
the tock board being programmed must be specified:

```bash
$ cd examples/blink/
$ make TOCK_BOARD=imix program
```

This builds and loads only a single app. Tock is capable of running multiple apps
concurrently:

Use `tockloader install -a 0x40000` to add additional apps, and
`tockloader list -a 0x40000` to see the list of installed applications. The `-a`
flag specifies the address of the application space, which is different between
boards.

Please note that forgetting to specify `TOCK_BOARD=imix` when using `make program`
or forgetting to specify `-a 0x40000` when using `tockloader install` can result
in overwriting a portion of the kernel, which should be fixed by flashing the
kernel again.

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

Connect to the FTDI chip by plugging a USB cable into the DBG\_USB port (the
one closer to the middle), and then use `miniterm.py` to open that serial port:

```bash
$ miniterm.py --dtr 0 --rts 1 /dev/ttyUSB0 115200
```

or

```bash
tockloader listen
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

