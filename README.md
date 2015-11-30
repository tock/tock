# Tock Embedded OS [![Build Status](https://travis-ci.org/helena-project/tock.svg?branch=master)](https://travis-ci.org/helena-project/tock)

[![Join the chat at https://gitter.im/helena-project/tock](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/helena-project/tock?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

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

1. [Rust](http://www.rust-lang.org/) 1.4.0-nightly (e35fd7481 2015-08-17)
2. [arm-none-eabi toolchain](https://launchpad.net/gcc-arm-embedded/) (version >= 4.9)
3. stormloader (recommended) or JLinkExe for programming the storm
4. Command line utilities: wget, sed, make

### Installing Requirements

### Rust (nightly)

We are using `rustc 1.4.0-nightly (e35fd7481 2015-08-17)`:

```bash
$ curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly --date=2015-08-18
```

#### `arm-none-eabi` toolchain

On Mac OS X, you can get the arm-none-eabi toolchain via port:

```bash
$ port install arm-none-eabi-gcc
```

or via homebrew:

```bash
$ brew tap PX4/homebrew-px4
$ brew update
$ brew install gcc-arm-none-eabi-49
```

On Linux it is available through many distribution managers:

```bash
$ pacman -S arm-none-eabi-gcc
$ apt-get install gcc-arm-none-eabi
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

Simply call make:

```bash
make
```

## Programming the storm

If you are using the stormloader, there is a `make` rule that compiles the
source and programs the storm:

```bash
make program
```

If you are using `JLinkExe`, use the script included in the source root
directory:

```bash
JLinkExe prog.jlink
```

## Printf support

To get the UART printf from firestorm:

```bash
sload tail -i
```


