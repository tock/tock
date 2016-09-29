Platform-Specific Instructions: Storm
=====================================

The [Firestorm](http://storm.rocks/ref/firestorm.html) is a platform from UC
Berkeley's [Software Defined Buildings](http://sdb.cs.berkeley.edu/sdb/)
research group. It is based on the Atmel SAM4L and includes an RF233 802.15.4
radio, a Nordic NRF51822 BLE radio, a light sensor, accelerometer and
temperature sensor.

To program the Tock kernel and apps onto the Firestorm, you need to have the
stormloader python library installed. This requires 
libftdi to be installed as well. (see [below](#stormloader) for detailed
instructions):

```bash
$ pip install stormloader
```

You also need to add the following udev rule for the Firestorm's FTDI chip:

```bash
sudo su
echo 'ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6015", MODE="0666"' > /etc/udev/rules.d/99-storm.rules
```

## Programming the kernel

To program the Tock kernel onto the Firestorm, `cd` into the `boards/storm` directory and run:

```bash
$ make program
```

This will build `boards/storm/target/target/storm.elf`, generate a "Storm Drop
Binary" file, and program it on the storm using the `stormloader`.

The Tock kernel can also be flashed over JTAG using `make flash`.

## Programming apps

All user-level code lives in the `userland` subdirectory. This includes a
specially compiled version of newlib, a user-level library for talking to the
kernel and specific drivers and a variety of example applications.

Userland compilation units are specific to a particular architecture (e.g.
`cortex-m4`, `cortex-m0`) since the compiler emits slightly different code for
each variant, but is portable across boards with the same drivers. The `TOCK_ARCH`
environment variable controls which architecture to compile to. You can set the
`TOCK_ARCH` to any architecture GCC's `-mcpu` option accepts. By default, `TOCK_ARCH`
is set to `cortex-m4` for the `storm` board.

To compile an app, `cd` to the desired app and `make`. For example:

```bash
$ cd userland/examples/blink/
$ make
```

This will build the app and generate a binary in Tock Binary Format (using the
`elf2tbf` utility): `userland/examples/blink/build/cortex-m4/app.bin`. This
binary should either be programmed separately from the kernel. See the README
file in each board subdirectory for details.

Apps can be built and automatically uploaded from the root directory of Tock.

```bash
$ make examples/blink
```

Like the kernel, apps can be uploaded with `make program` or `make flash`.
```bash
$ cd userland/examples/blink/
$ make program
```

This builds and loads only a single app. Tock is capable of running multiple apps
concurrently. In order to load multiple apps, you can use the application upload
tools manually. They are located in `userland/tools/`, are separated by upload method
(`flash` or `program`) and take `.bin` files as input arguments.

Example

```bash
$ make -C userland/examples/blink
$ make -C userland/examples/c_hello
$ userland/tools/program/storm.py userland/examples/blink/build/cortex-m4/app.bin userland/examples/c_hello/build/cortex-m4/app.bin
```

## Console support

To access the console UART from the Firestorm use the `sload tail` subcommand:

```bash
sload tail
```

This will restart the storm and print console output to the terminal. To avoid
restarting, add the `-n` (for "**_n**o restart") command line flag:

```bash
sload tail -n
```

To forward terminal _input_ to the Firestorm, add the `-i` (for "**i**nteractive")
command line flag:

```bash
sload tail -i
```

## JTAG Programming

To connect to the board with a j-link programmer:

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

## Stormloader

You'll need to install
[libftdi-0.20.tar.gz](http://www.intra2net.com/en/developer/libftdi/download/libftdi-0.20.tar.gz)
for stormloader to function; newer versions will not work. In turn, libftdi
depends on libusb and libusb-config. On OS X, you can satisfy the libftdi
prereqs via homebrew:

```bash
brew install libusb libusb-compat
```

On Ubuntu you can satisfy this requirement by installing libusb and libftdi
(tested on Ubuntu 16.04).

```bash
sudo apt-get install libusb-1.0-0-dev
sudo apt-get install libftdi-dev
```

You can obtain stormloader via pip (python2 only, currently):

```bash
sudo pip install stormloader
```

Note that you may need to execute this command with the sudo -H option.

```bash
sudo -H pip install stormloader
```

You can update stormloader via pip as well:

```bash
sudo pip install -U stormloader
```

Then add a udev rule (Ubuntu) for the FTDI chip:

```bash
sudo su
echo 'ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6015", MODE="0666"' > /etc/udev/rules.d/99-storm.rules
```

