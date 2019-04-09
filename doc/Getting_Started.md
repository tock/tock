Tock Getting Started Guide
==========================

This covers how to install the toolchain on your platform to start using and
developing Tock.

## Requirements

1. [Rust](http://www.rust-lang.org/)
2. [rustup](https://rustup.rs/) to install Rust (version >= 1.11.0)
3. Command line utilities: make

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

We are using `nightly-2019-04-11`. We require
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
$ rustup install nightly-2019-04-11
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
(Linux): sudo pip3 install --upgrade tockloader
(MacOS): pip3 install --upgrade tockloader
```

## Compiling the Kernel

Tock builds a unique kernel for every _board_ it supports. Boards include
details like pulling together the correct chips and pin assignments. To
build a kernel, first choose a board, then navigate to that board directory.
e.g. `cd boards/hail ; make`.

Some boards have special build options that can only be used within the board's
directory.  All boards share a few common targets:

  - `all` (default): Compile Tock for this board.
  - `debug`: Generate build(s) for debugging support, details vary per board.
  - `doc`: Build documentation for this board.
  - `clean`: Remove built artifacts for this board.
  - `flash`: Load code using JTAG, if available.
  - `program`: Load code using a bootloader, if available.

The READMEs in each board provide more details for each platform.

## Compiling applications

All user-level code lives in separate repositories:

- [libtock-c](https://github.com/tock/libtock-c): C and C++ apps.
- [libtock-rs](https://github.com/tock/libtock-rs): Rust apps.

Compiled applications are architecture-specific (e.g. `cortex-m4`,
`cortex-m0`) since the compiler emits slightly different instructions
for each variant. Compiled applications can also depend on specific
drivers, which not all boards provide; if you load an application onto
a board that does not support every driver/system call it uses, some
system calls with return error codes (`ENODEVICE` or `ENOSUPPORT`).

Applications are built for all architectures Tock supports. Boards select an
appropriate architecture when uploading code (e.g. `cortex-m4` for the SAM4L on
the `imix` board). Apps are packaged into .tab files that contain compiled
binaries for all supported architectures.

## Loading the kernel and applications onto a board

To load a kernel onto a board using a serial bootloader, run

    $ make program

in the board's directory. To load the kernel using JTAG, run

    $ make flash

Tockloader can help with installing a test app. For example, to install
the `blink` app, simply run:

    $ tockloader install blink

This will fetch it from the TockOS app repository and load it onto the board.

### Optional Requirements

Some boards in Tock support other tools to load code and debug.

#### `openocd`

Works with various JTAG debuggers. We require at least version `0.8.0` to
support the SAM4L on `imix`.

```bash
(Linux): sudo apt-get install openocd
(MacOS): brew install open-ocd
```

#### `JLinkExe`

If you want to upload code through a [JLink JTAG
debugger](https://www.segger.com/j-link-edu.html) (available on
[Digikey](https://www.digikey.com/product-detail/en/segger-microcontroller-systems/8.08.90-J-LINK-EDU/899-1008-ND/2263130)), you should install JLinkExe. We require a version greater than or equal to `5.0`.

It is available [here](https://www.segger.com/downloads/jlink). You want to
install the "J-Link Software and Documentation Pack". There are various
packages available depending on operating system.

### Loading code onto a board

This is generally done with `make program` and `make flash`, but is board
specific. To learn how to program your specific hardware, please see
the [board-specific READMEs](../boards/README.md).


## Formatting Rust source code

Rust includes a tool for automatically formatting Rust source
code. Simply run:

    $ make format

from the root of the repository to format all rust code in the repository.

## Keeping build tools up to date

Occasionally, Tock updates to a new nightly version of Rust. The build system
automatically checks whether the versions of `rustc` and `rustup` are correct
for the build requirements, and updates them when necessary. After the
installation of the initial four requirements, you shouldn't have to worry
about keeping them up to date.
