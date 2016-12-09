Tock Getting Started Guide
==========================

This covers how to get the toolchain setup on your platform to start using and
developing Tock.

## Requirements

1. [Rust](http://www.rust-lang.org/) (nightly)
2. [arm-none-eabi toolchain](https://launchpad.net/gcc-arm-embedded/) (version >= 5.0)
3. stormloader (recommended) or JLinkExe for programming the storm
4. Command line utilities: wget, sed, make

### Installing Requirements

#### Rust (nightly)

We are using `rustc 1.12.0-nightly (54c0dcfd6 2016-07-28)`. We recommend
installing it with [rustup](http://www.rustup.rs) so you can manage multiple
versions of Rust and continue using stable versions for other Rust code:

```bash
$ curl https://sh.rustup.rs -sSf | sh
```

This will install `rustup` in your home directory, so you will need to
source `~/.profile` or open a new shell to add the `.cargo/bin` directory
to your `$PATH`.

Then override the default version of Rust to use for Tock by running the
following from the top-level Tock directory:

```bash
$ rustup override set nightly-2016-07-29
```

#### `arm-none-eabi` toolchain

We are currently using arm-none-eabi-gcc version 5.4 from the gcc-arm-embedded
PPA on launchpad. Using pre-5.0 versions from that repo, or other versions
packaged with a newlib version earlier than 2.3 will run into problems with
missing ARM intrinsics (e.g., `__aeabi_memclr`).

##### Mac OS X

With [MacPorts](https://www.macports.org/):

```bash
$ port install arm-none-eabi-gcc
```

or with [Homebrew](http://brew.sh/):

```bash
$ brew tap PX4/homebrew-px4
$ brew update
$ brew install gcc-arm-none-eabi
```

##### Linux

On Linux we recommend getting packages from the [Launchpad repo](https://launchpad.net/gcc-arm-embedded/+download).

###### Compiled Binaries

```bash
$ curl https://launchpad.net/gcc-arm-embedded/5.0/5-2016-q2-update/+download/gcc-arm-none-eabi-5_4-2016q2-20160622-linux.tar.bz2
```

###### Ubuntu

```bash
$ sudo add-apt-repository ppa:team-gcc-arm-embedded/ppa
$ sudo apt-get update
$ sudo apt-get install gcc-arm-embedded
```

###### Arch

On Arch Linux the `arm-none-eabi` package in pacman contains a sufficiently up
to date version of newlibc.

##### Windows

For Windows and other operating systems, download site is
[here](https://launchpad.net/gcc-arm-embedded/+download).

##### Other

Alternatively, if you would like simulator mode in `arm-none-eabi-gdb`,
you can use the build scripts in the `tools` directory, in this order:
`build-arm-binutils` then `build-arm-gcc` then `build-arm-gdb`.

## Building the Kernel

To build the kernel, just type `make` in the root directory. To upload code to
a board, type `make program`.

The root Makefile selects a board and architecture to build the kernel for and
routes all calls to that board's specific Makefile. The root Makefile is set up
with the following defaults:

```
TOCK_BOARD ?= storm
TOCK_ARCH ?= cortex-m4
```

To build for a different platform, multiple options exist:

 * You can add an environment variable for the `TOCK_BOARD` and `TOCK_ARCH`.
    `TOCK_BOARD` is the directory name inside `boards/`.
    `TOCK_ARCH` is the gcc architecture name. Ex: `cortex-m4` or `cortex-m0`.

    ```bash
    $ make TOCK_BOARD=nrf51dk
    ```

 * You can also build the kernel for a specific board by entering the board's directory

    ```bash
    $ cd boards/nrf51dk/
    $ make
    ```

Board specific Makefiles are located in `boards/<BOARD>/`. Some boards have
special build options that can only be used within the board's directory.
Generic options such as `clean`, `doc`, `debug`, `program`, and `flash` can be
accessed from Tock's root

To upload code to a board, use the `program` or `flash` options. `program`
uploads code over a serial bootloader. `flash` uploads code over JTAG. Not all
platforms support all methods of code upload.


## Building apps

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


## Board-Specific Instructions

For instructions on building, uploading code, and debugging on specific
boards, see board specific READMEs.

 * [Storm](boards/storm/README.md)
 * [nRF](boards/nrf51dk/README.md)


## Formatting Rust Source Code

Rust includes a tool for automatically formatting Rust source code. This requires
a `cargo` tool:

    $ cargo install rustfmt

Then run:

    $ make format

to format the repository.