imix: Platform-Specific Instructions
=====================================

Kernel and userland software can be flashed onto the imix using
[openocd](http://openocd.org/).


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

Userland compilation units are specific to a particular architecture (e.g.
`cortex-m4`, `cortex-m0`) since the compiler emits slightly different code for
each variant, but is portable across boards with the same drivers. The `TOCK_ARCH`
environment variable controls which architecture to compile to. You can set the
`TOCK_ARCH` to any architecture GCC's `-mcpu` option accepts. By default, `TOCK_ARCH`
is set to `cortex-m4` for the `imix` board.

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
concurrently. *TODO*

## Console support

*TODO*

## JTAG Programming

To connect to the board with a J-Link programmer:

```bash
JLinkExe -device ATSAM4LC8C -speed 1200 -if swd -AutoConnect 1
```

To debug with GDB:

```bash
JLinkGDBServer -device ATSAM4LC8C -speed 1200 -if swd -AutoConnect 1 -port 2331

(open a new terminal)

arm-none-eabi-gdb <ELF_FILE>
```

You also need a `.gdbinit` file:

```bash
target remote localhost:2331
load
mon reset
break main
```

