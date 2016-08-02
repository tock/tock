# Tock Embedded OS [![Build Status](https://travis-ci.org/helena-project/tock.svg?branch=master)](https://travis-ci.org/helena-project/tock)

Tock is an operating system designed for running multiple concurrent, mutually
distrustful applications on Cortex-M based embedded platforms like the
[Storm](http://storm.rocks). Tock's design centers around protection, both from
potentially malicious applications and from device drivers. Tock uses two
mechanisms to protect different components of the operating system. First, the
kernel and device drivers are written in Rust, a systems programming language
that provides compile-time memory safety, type safety and strict aliasing. Tock
uses Rust to protect the kernel (e.g. the scheduler and hardware abstraction
layer) from platform specific device drivers as well as isolate device drivers
from each other. Second, Tock uses memory protection units to isolate
applications from each other and the kernel.

## Requirements

1. [Rust](http://www.rust-lang.org/) (nightly)
2. [arm-none-eabi toolchain](https://launchpad.net/gcc-arm-embedded/) (version >= 4.9)
3. stormloader (recommended) or JLinkExe for programming the storm
4. Command line utilities: wget, sed, make

### Installing Requirements

### Rust (nightly)

We are using `rustc 1.12.0-nightly (54c0dcfd6 2016-07-28)`. Install it using rustup:

```bash
$ curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --date=2016-07-29
```

Alternatively, you can use [multirust](https://github.com/brson/multirust):

```bash
$ curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh
$ multirust override nightly-2016-07-29
```

#### `arm-none-eabi` toolchain

We are currently using arm-none-eabi-gcc 5.4. Using pre-5.0 versions may
run into problems with missing intrinsics (e.g., ```__aeabi_memclr```). 

On Mac OS X, you can get the arm-none-eabi toolchain via port:

```bash
$ port install arm-none-eabi-gcc
```

or via homebrew:

```bash
$ brew tap PX4/homebrew-px4
$ brew update
$ brew install gcc-arm-none-eabi-54
```

On Linux we recommend getting packages from Launchpad

https://launchpad.net/gcc-arm-embedded/+download

E.g.:

```bash
$ curl https://launchpad.net/gcc-arm-embedded/5.0/5-2016-q2-update/+download/gcc-arm-none-eabi-5_4-2016q2-20160622-linux.tar.bz2
```

For Windows and other operating systems, download site is
[here](https://launchpad.net/gcc-arm-embedded/+download).

Alternatively, if you would like simulator mode in `arm-none-eabi-gdb`,
you can use the build scripts in the `tools` directory, in this order:
`build-arm-binutils` then `build-arm-gcc` then `build-arm-gdb`.

### Stormloader

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

## Building

If all the tools are in your `$PATH`, you should be good to go. Otherwise set the env variables:

* `RUSTC` - `rustc` compiler
* `CC` - `arm-none-eabi-gcc`
* `OBJCOPY` - `arm-none-eabi-objcopy`

The build system respects the environment variable `TOCK_PLATFORM`, which
currently defaults to the `storm` platform but can be set to any available
platform (anything in `src/platform`).

Invoking `make` in the root directory will build the tock kernel (placing it
in `tock/build/$(TOCK_PLATFORM)/`).

To build applications, change to `apps/$(APP)/` directory and invoke `make`.
As applications depend on the kernel, they will ensure it is up to date first.
It will then build both the application (into `tock/build/$(TOCK_PLATFORM)/$(APP)/$(APP).elf`)
and a loadable image of the tock kernel with that single application installed
as a convenience (`tock/build/$(TOCK_PLATFORM)/$(APP)/kernel_and_app.elf`).

Most platforms also define the convenience target `make program` that will load
the `kernel_and_app` image by default.

## Programming the storm

If you are using the stormloader, there is a `make` rule that compiles the
source and programs the storm:

```bash
make program
```

## Printf support

To get the UART printf from firestorm:

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

