The [Firestorm](http://storm.rocks/ref/firestorm.html) is a platform from UC
Berkeley's [Software Defined Buildings](http://sdb.cs.berkeley.edu/sdb/)
research group. It is based on the Atmel SAM4L and includes an RF233 802.15.4
radio, a Nordic NRF51822 BLE radio, a light sensor, accelerometer and
temperature sensor.

To program the Tock kernel and apps onto the Firestorm, you need to have the
stormloader python library installed (see [below](#stormloader) for detailed
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

To program the Tock kernel onto the Firestorm, run:

```bash
$ make TOCK_PLATFORM=storm program
```

This will build `build/storm/kernel.elf`, generate a "Storm Drop Binary"
(`build/storm/kernel.sdb`) file, and program it on the storm using the `stormloader`.

## Programming user-level processes

Apps are programmed to the Firestorm independently of the kernel, using the
Python script in `tools/program/firestorm.py`. The programming utility takes a
list binaries in Tock Binary Format (tbf), generated from `elf2tbf`.

To build an application, simply run `make` from the application's directory
(`make`'s `-C` flag tells it which directory to build from):

```bash
$ make -C apps/blink
```

This will generate the file `build/storm/blink/blink.bin`, which you can then
pass to the program utility:

```bash
$ tools/program/firestorm.py build/storm/blink/blink.bin
```

You can pass multiple binaries to program multiple apps:

```bash
$ tools/program/firestorm.py build/storm/blink/blink.bin build/storm/sensors/sensors.bin
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

You can obtain stormloader via pip (python2 only, currently):

```bash
sudo pip install stormloader
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
