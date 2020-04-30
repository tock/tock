Tock Getting Started Guide
==========================

This covers how to install the toolchain on your platform to start using and
developing Tock.

<!-- npm i -g markdown-toc; markdown-toc -i Getting_Started.md -->

<!-- toc -->

- [Requirements](#requirements)
  * [Super Quick Setup](#super-quick-setup)
  * [Installing Requirements](#installing-requirements)
    + [Rust (nightly)](#rust-nightly)
    + [Tockloader](#tockloader)
- [Compiling the Kernel](#compiling-the-kernel)
- [Loading the kernel onto a board](#loading-the-kernel-onto-a-board)
  * [Installing `JLinkExe`](#installing-jlinkexe)
  * [Installing `openocd`](#installing-openocd)
  * [(Linux): Adding a `udev` rule](#linux-adding-a-udev-rule)
- [Installing your first application](#installing-your-first-application)
- [Compiling applications](#compiling-applications)
- [Developing TockOS](#developing-tockos)
  * [Formatting Rust source code](#formatting-rust-source-code)
  * [Keeping build tools up to date](#keeping-build-tools-up-to-date)

<!-- tocstop -->

## Requirements

1. [Rust](http://www.rust-lang.org/)
2. [rustup](https://rustup.rs/) to install Rust (version >= 1.11.0)
3. Command line utilities: make
4. A supported board or QEMU configuration.

   If you are just starting to work with TockOS, you should look in
   the [`boards/` subdirectory](../boards/README.md) and choose one of
   the options with `tockloader` support to load applications, as that
   is the configuration that most examples and tutorials assume.

   * Info about testing Tock on QEMU
     * 01/08/2020 : Among the boards supported by Tock, [SiFive HiFive1 RISC-V Board](../boards/hifive1/#running-in-qemu) can be tested in QEMU.

### Super Quick Setup

Nix:
```
$ nix-shell
```

MacOS:
```
$ curl https://sh.rustup.rs -sSf | sh
$ pip3 install --upgrade tockloader
```

Ubuntu:
```
$ curl https://sh.rustup.rs -sSf | sh
$ pip3 install --upgrade tockloader --user
$ grep -q dialout <(groups $(whoami)) || sudo usermod -a -G dialout $(whoami) # Note, will need to reboot if prompted for password
```

Then build the kernel by running `make` in the `boards/<platform>` directory.

### Installing Requirements

These steps go into a little more depth. Note that the build system is capable
of installing some of these tools, but you can also install them yourself.

#### Rust (nightly)

We are using `nightly-2020-04-30`. We require
installing it with [rustup](http://www.rustup.rs) so you can manage multiple
versions of Rust and continue using stable versions for other Rust code:

```bash
$ curl https://sh.rustup.rs -sSf | sh
```

This will install `rustup` in your home directory, so you will need to
source `~/.profile` or open a new shell to add the `.cargo/bin` directory
to your `$PATH`.

Then install the correct nightly version of Rust:

```bash
$ rustup install nightly-2020-04-30
```

#### Tockloader

`tockloader` programs the kernel and applications onto boards, and also has
features that are generally useful for all Tock boards, such as easy-to-manage
serial connections, along with the ability to list, add, replace, and remove
applications over JTAG (or USB if a bootloader is installed).

1. [tockloader](https://github.com/tock/tockloader) (version >= 1.0)

Tockloader is a Python application and can be installed with the Python
package manager (pip).

```bash
(Linux): pip3 install --upgrade tockloader --user
(MacOS): pip3 install --upgrade tockloader
```

## Compiling the Kernel

Tock builds a unique kernel for every _board_ it supports. Boards include
details like pulling together the correct chips and pin assignments. To
build a kernel, first choose a board, then navigate to that board directory.
e.g. `cd boards/nordic/nrf52840dk ; make`.

Some boards have special build options that can only be used within the board's
directory.  All boards share a few common targets:

  - `all` (default): Compile Tock for this board.
  - `debug`: Generate build(s) for debugging support, details vary per board.
  - `doc`: Build documentation for this board.
  - `clean`: Remove built artifacts for this board.
  - `flash`: Load code using JTAG, if available.
  - `program`: Load code using a bootloader, if available.

The [board-specific READMEs](../boards/README.md) in each board's
subdirectory provide more details for each platform.

## Loading the kernel onto a board

The process to load the kernel onto the board depends on the board.
There are two main variants: some boards (notably the
[Imix](../boards/imix/README.md) and [Hail](../boards/hail/README.md)
boards) have a serial bootloader, most other boards use a programming
adapter that supports the JTAG or SWD protocol instead.

To load a kernel onto a board using a serial bootloader, no other
software is required and you can just run

    $ make program

in the board's directory. To load the kernel using a programming
adapter, you need the appropriate software that supports the adapter
and can then install the kernel by running

    $ make flash

Depending on the adapter, you will need either the free `openocd` or
Segger's proprietary `JLinkExe`. Programming adapters are available as
standalone devices (for example the [JLink EDU JTAG
debugger](https://www.segger.com/j-link-edu.html) available on
[Digikey](https://www.digikey.com/product-detail/en/segger-microcontroller-systems/8.08.90-J-LINK-EDU/899-1008-ND/2263130)),
but most development boards come with an onboard programming and
debugging adapter. In that case, the board you use determines which
software you will need and the `Makefile` in the board directory will
know which one to call. Again, the [board-specific
READMEs](../boards/README.md) provide the required details.

### Installing `JLinkExe`

`JLink` is available [from the Segger
website](https://www.segger.com/downloads/jlink). You want to install
the "J-Link Software and Documentation Pack". There are various
packages available depending on operating system. We require a version
greater than or equal to `5.0`.

### Installing `openocd`

`Openocd` works with various programming and debugging adapters. For
most purposes, available distribution packages are sufficient and it can
be installed with:

```bash
(Linux/Debian): sudo apt-get install openocd
(MacOS): brew install open-ocd
```

We require at least version `0.8.0` to support the SAM4L on `imix` if
you choose to flash it using an adapter instead of the bootloader.
Some boards (at the time of writing the HiFive1 RISC-V board) may
require newer or unreleased versions, in that case you should follow
the installation instructions on the [`openocd`
website](http://openocd.org/getting-openocd/).

### (Linux): Adding a `udev` rule

Depending on which programming adapter you use, you may want to add a
`udev` rule in `/etc/udev/rules.d` that allows you to interact with
the board as a user instead of as root. If you install the `deb`
packet of the `JLink` software it will automatically install a
`/etc/udev/rules.d/99-jlink.rules` that allows everyone to access the
adapter. If you use something else, like for example the onboard
programmer of a ST Nucleo board, you could install something like this as
`/etc/udev/rules.d/99-stlinkv2-1.rules`:

```
# stm32 nucleo boards, with onboard st/linkv2-1
# ie, STM32F0, STM32F4.
# STM32VL has st/linkv1, which is quite different

SUBSYSTEMS=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", \
    MODE:="0660", GROUP="dialout", \
    SYMLINK+="stlinkv2-1_%n"
```

## Installing your first application

A kernel alone isn't much use, as an embedded developer you want to
see some LEDs blink.  Fortunately, there is an example `blink` app
available from the TockOS app repository which `tockloader` can
download and install (if you are using a board that is supported
by `tockloader`).

For certain boards (e.g. Hail and imix), `tockloader` can read
attributes from the board to configure how it communicates with the
board. For many boards, however, `tockloader` cannot know
which board and communication method you want to use, so you have to
tell it explicitly. For example:

```bash
$ tockloader install --board nrf52dk --jlink blink
Could not find TAB named "blink" locally.

[0]     No
[1]     Yes

Would you like to check the online TAB repository for that app?[0] 1
Installing apps on the board...
Using known arch and jtag-device for known board nrf52dk
Finished in 2.567 seconds
```

Boards that use `openocd` will of course require the parameter
`--openocd` instead of `--jlink`.  If your board has a serial
bootloader, `tockloader` should work without any additional arguments:

    $ tockloader install blink

However, you can specify the board type manually as well:

    $ tockloader install --board imix blink

You can also tell it which serial port to use (which is useful if you
have multiple boards plugged in) by passing it the `--port` parameter
like `--port /dev/ttyACM0` to use `/dev/ttyACM0`:

    $ tockloader --port /dev/ttyACM0 install blink

To see the list of boards `tockloader` knows about you can run:

    $ tockloader list-known-boards

If everything has worked until here, the LEDs on your board should now
display a binary counter. Congratulations, you have a working TockOS
installation on your board!

## Compiling applications

The last remaining step is to compile applications locally.
All user-level code lives in two separate repositories:

- [libtock-c](https://github.com/tock/libtock-c): C and C++ apps.
- [libtock-rs](https://github.com/tock/libtock-rs): Rust apps.

The C version of the Tock library and the example applications is older and more
stable, so it is a good idea to look at these first. So look at the [libtock-c
README](https://github.com/tock/libtock-c/blob/master/README.md) and follow the
steps therein.  Then you can do the same for the [libtock-rs
README](https://github.com/tock/libtock-rs/blob/master/README.md).  This should
give you a first impression of how to build and deploy applications for TockOS.

For an introduction on how applications work in TockOS, have a look at
the ["Userland" document](Userland.md) in this directory.

## Developing TockOS

### Formatting Rust source code

Rust includes a tool for automatically formatting Rust source
code. Simply run:

    $ make format

from the root of the repository to format all rust code in the repository.

### Keeping build tools up to date

Occasionally, Tock updates to a new nightly version of Rust. The build system
automatically checks whether the versions of `rustc` and `rustup` are correct
for the build requirements, and updates them when necessary. After the
installation of the initial four requirements, you shouldn't have to worry
about keeping them up to date.
