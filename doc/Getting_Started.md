Tock Getting Started Guide
==========================

This covers how to install the toolchain on your platform to start using and
developing Tock.

<!-- npm i -g markdown-toc; markdown-toc -i Getting_Started.md -->

<!-- toc -->

- [Super Quick Setup to Build the Tock Kernel](#super-quick-setup-to-build-the-tock-kernel)
- [Detailed Setup for Building the Tock Kernel](#detailed-setup-for-building-the-tock-kernel)
  * [Installing Requirements](#installing-requirements)
    + [Rust (nightly)](#rust-nightly)
  * [Compiling the Kernel](#compiling-the-kernel)
- [Hardware and Running Tock](#hardware-and-running-tock)
  * [Tockloader](#tockloader)
  * [Programming Adapter](#programming-adapter)
    + [Installing `JLinkExe`](#installing-jlinkexe)
    + [Installing `openocd`](#installing-openocd)
    + [Installing `probe-rs`](#installing-probe-rs)
  * [Loading the Kernel onto a Board](#loading-the-kernel-onto-a-board)
- [Installing Applications](#installing-applications)
  * [Compiling Your Own Applications](#compiling-your-own-applications)
- [Developing TockOS](#developing-tockos)
  * [Formatting Rust source code](#formatting-rust-source-code)
  * [Keeping build tools up to date](#keeping-build-tools-up-to-date)

<!-- tocstop -->

## Super Quick Setup to Build the Tock Kernel

If you just want to get started quickly, follow these steps for your
environment:

MacOS:
```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ pipx install tockloader
$ pipx ensurepath
```

Ubuntu:
```
$ sudo apt install -y build-essential python3-pip curl
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ pipx install tockloader
$ pipx ensurepath
$ grep -q dialout <(groups $(whoami)) || sudo usermod -a -G dialout $(whoami) # Note, will need to reboot if prompted for password
```

Nix:
```
$ nix-shell
```

Then build the kernel by running `make` in the `boards/<platform>` directory.


## Detailed Setup for Building the Tock Kernel

To build the Tock kernel, you will need:

1. [Rust](http://www.rust-lang.org/)
2. [rustup](https://rustup.rs/) (version >= 1.23.0) to install Rust
3. Host toolchain (gcc, glibc)
4. Command line utilities: make, find

### Installing Requirements

These steps go into a little more depth. Note that the build system is capable
of installing some of these tools, but you can also install them yourself.

#### Rust (nightly)

We are using `nightly-2025-05-19`. We require
installing it with [rustup](http://www.rustup.rs) so you can manage multiple
versions of Rust and continue using stable versions for other Rust code:

```bash
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This will install `rustup` in your home directory, so you will need to
source `~/.profile` or open a new shell to add the `.cargo/bin` directory
to your `$PATH`.

Then install the correct nightly version of Rust:

```bash
$ rustup install nightly-2025-05-19
```

### Compiling the Kernel

Tock builds a unique kernel for every _board_ it supports. Boards include
details like pulling together the correct chips and pin assignments. To build a
kernel, first choose a board, then navigate to that board directory. e.g. `cd
boards/nordic/nrf52840dk ; make`.

Some boards have special build options that can only be used within the board's
directory. All boards share a few common targets:

- `all` (default): Compile Tock for this board.
- `debug`: Generate build(s) for debugging support, details vary per board.
- `doc`: Build documentation for this board.
- `clean`: Remove built artifacts for this board.
- `install`: Load the kernel onto the board.

The [board-specific READMEs](../boards/README.md) in each board's subdirectory
provide more details for each platform.


## Hardware and Running Tock

To run the Tock kernel you need:

1. A supported board or QEMU configuration
2. Tockloader
3. A programming adapter for loading code (required for most boards)

If you are just starting to work with TockOS, you should look in the [`boards/`
subdirectory](../boards/README.md) and choose one of the options with
`tockloader` support to load applications, as that is the configuration that
most examples and tutorials assume.

If you do not have a supported hardware board, Tock has some limited support for
running the kernel in [QEMU](https://www.qemu.org/). As of 01/08/2020, the
[SiFive HiFive1 RISC-V Board](../boards/hifive1/#running-in-qemu) can be tested
in QEMU.

> **Note:** QEMU support in Tock is in the early stages. Please be sure to check
> whether and how QEMU is supported for a board based on the table in the
> [`boards/` subdirectory](../boards/README.md). The `make ci-job-qemu` target
> is the authority on QEMU support.

### Tockloader

[tockloader](https://github.com/tock/tockloader) programs the kernel and
applications onto boards, and also has features that are generally useful for
all Tock boards, such as easy-to-manage serial connections, along with the
ability to list, add, replace, and remove applications over JTAG (or USB if a
bootloader is installed).

Tockloader is a Python application and can be installed with the Python package
manager for executables (pipx):

```bash
$ pipx install tockloader
$ pipx ensurepath
```

### Programming Adapter

For some boards, you will need a programming adapter to flash code. Check the
"Interface" column in the [boards README](../boards/README.md) for the board you
have to see what the default programming adapter you need is. There are
generally four options:

1. `Bootloader`: This means the board supports the Tock bootloader and you do
   not need any special programming adapter. Tockloader has built-in support.
2. `jLink`: This is a proprietary tool for loading code onto microcontrollers
   from Segger. You will need to install this if you do not already have it. See
   the instructions below.
3. `openocd`: This is a free programming adapter which you will need to install
   if you do not already have it. See the instructions below.
4. `probe-rs`: This is a programming and debugging tool written in Rust. You will
   need to install this if you do not already have it. See the instructions
   below.
5. `custom`: The board uses some other programming adapter, likely a
   microcontroller-specific tool. See the board's README for how to get started.

#### Installing `JLinkExe`

`JLink` is available [from the Segger
website](https://www.segger.com/downloads/jlink). You want to install the
"J-Link Software and Documentation Pack". There are various packages available
depending on operating system. We require a version greater than or equal to
`5.0`.

#### Installing `openocd`

`Openocd` works with various programming and debugging adapters. For most
purposes, available distribution packages are sufficient and it can be installed
with:

```bash
(Ubuntu): sudo apt-get install openocd
(MacOS): brew install open-ocd
(Fedora): sudo dnf install openocd
```

We require at least version `0.10.0`.

#### Installing `probe-rs`

[`probe-rs`](https://probe.rs/) works with various programming and debugging adapters. It can be
installed with:

```bash
(Ubuntu): curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
(MacOS): brew tap probe-rs/probe-rs && brew install probe-rs
(Windows) irm https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.ps1 | iex
```

Or generally, follow the [`probe-rs` installation instructions in their own documentation](https://probe.rs/docs/getting-started/installation/).

### Loading the Kernel onto a Board

The process to load the kernel onto the board depends on the board. You should
be able to program the kernel by changing to the correct board directory in
`tock/boards/` and running:

```
$ make install
```

Each board has a default programming adapter tool for flashing code. Some boards
only support a single tool, while others support multiple. You can inspect the
`Makefile` within the board folder to see which flashing options exist.

## Installing Applications

A kernel alone isn't much use, as an embedded developer you want to see some
LEDs blink. Fortunately, there is an example `blink` app available from the
TockOS app repository which `tockloader` can download and install.

To install blink, run:

```
$ tockloader install blink
```

Tockloader will automatically detect your board, download the app from the Tock
website, and flash it for you.

If everything went well, the LEDs on your board should now display a binary
counter. Congratulations, you have a working TockOS installation on your board!

### Compiling Your Own Applications

You can also compile applications locally. All user-level code lives in two
separate repositories:

- [libtock-c](https://github.com/tock/libtock-c): C and C++ apps.
- [libtock-rs](https://github.com/tock/libtock-rs): Rust apps.

You can use either version by following the steps in their respective READMEs:
[libtock-c README](https://github.com/tock/libtock-c/blob/master/README.md) and
[libtock-rs README](https://github.com/tock/libtock-rs/blob/master/README.md).


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
installation of the initial four requirements, you shouldn't have to worry about
keeping them up to date.
