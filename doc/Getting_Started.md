Tock Getting Started Guide
==========================

This covers how to get the toolchain setup on your platform to start using and
developing Tock.

## Requirements

1. [Rust](http://www.rust-lang.org/) (nightly)
2. [arm-none-eabi toolchain](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads) (version >= 5.2)
3. [tockloader](https://github.com/helena-project/tockloader) (recommended) or [JLinkExe](https://www.segger.com/downloads/jlink)
4. Command line utilities: wget, sed, make

### Installing Requirements

#### Rust (nightly)

We are using `rustc 1.16.0-nightly (83c2d9523 2017-01-24)`. We recommend
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
$ cd tock
$ rustup override set nightly-2017-01-25
```

If you are having trouble with running the correct version of rust, you
can check the `$HOME/multirust/settings.toml` file for which versions
are set to which folders.

#### `arm-none-eabi` toolchain

We generally track the latest version of arm-none-eabi-gcc [as released by
ARM](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads).

There are known issues with arm-none-eabi-gcc version 5.1 and older, or other
versions packaged with a newlib version earlier than 2.3, as they will run into
problems with missing ARM intrinsics (e.g., `__aeabi_memclr`). Tock does not
support these versions.

##### Compiled Binaries

Pre-compiled binaries are available [from
ARM](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads).
The recommendations below will set up your operating system's package manager
to track the latest release from ARM.

##### Mac OS X

With [Homebrew](http://brew.sh/) (preferred):

```bash
$ brew tap ARMmbed/homebrew-formulae
$ brew update
$ brew install arm-none-eabi-gcc
```

or with [MacPorts](https://www.macports.org/):

```bash
$ port install arm-none-eabi-gcc
```

###### Heads Up!

The `make debug` target asks the Tock build system to generate a listings
(disassembly) file. Some developers have noticed that `arm-none-eabi-objdump`
takes a long time (order several minutes) on a mac while Activity Monitor
reports that `opendirectoryd` pegs the CPU.

This is a [known issue](http://superuser.com/questions/350879/) that you can
resolve by commenting out the `/home` line from `/etc/auto_master` and then
running `sudo automount -vc` to apply the changes.

##### Linux

If you install the binaries but get a "no such file or directory" error
when trying to run them, then you are most likely missing needed libraries.
Check that you have a 64-bit version of libc installed.

###### Ubuntu

```bash
$ sudo add-apt-repository ppa:team-gcc-arm-embedded/ppa
$ sudo apt-get update
$ sudo apt-get install gcc-arm-embedded
```

###### Arch

On Arch Linux the `arm-none-eabi-newlib` package in pacman contains a
sufficiently up-to-date version of newlibc.

```bash
$ sudo pacman -S arm-none-eabi-gcc arm-none-eabi-newlib arm-none-eabi-gdb
```

##### Windows

You can download precompiled binaries for Windows from the ARM site listed
above. While we expect things should work on Windows, none of the active Tock
developers currently develop on Windows, so it is possible that are some
unexpected pitfalls.

##### Other

Alternatively, if you would like simulator mode in `arm-none-eabi-gdb`,
you can use the build scripts in the `tools` directory, in this order:
`build-arm-binutils` then `build-arm-gcc` then `build-arm-gdb`.

## Compiling the Kernel

To build the kernel, just type `make` in the root directory.  The root
Makefile selects a board and architecture to build the kernel for and
routes all calls to that board's specific Makefile. The root Makefile
is set up with the following defaults:

```
TOCK_BOARD ?= storm
```

Thus it compiles for the storm board by default. There are two ways to
build for a different board:

 * You can compile the kernel for a specific board by running the command
   from inside the board's directory:

    ```bash
    $ cd boards/nrf51dk/
    $ make
    ```

 * Alternatively, you can add a `TOCK_BOARD` environment variable where
    `TOCK_BOARD` is the directory name inside `boards/`.

    ```bash
    $ make TOCK_BOARD=nrf51dk
    ```

Board specific Makefiles are located in `boards/<BOARD>/`. Some boards have
special build options that can only be used within the board's directory.
Generic options such as `clean`, `doc`, `debug`, `program`, and `flash` can be
accessed from Tock's root.

## Compiling applications

All user-level code lives in the `userland` subdirectory. This
includes a specially compiled version of newlib, a user-level library
for talking to the kernel and specific drivers and a variety of
example applications.

Compiled applications are architecture-specific (e.g.  `cortex-m4`,
`cortex-m0`) since the compiler emits slightly different instructions
for each variant. Compiled applications can also depend on specific
drivers, which not all boards provide; if you load an application onto
a board that does not support every driver/system call it uses, some
system calls with return error codes (`ENODEVICE` or `ENOSUPPORT`).

The `TOCK_ARCH` environment variable controls which chip architecture
to compile to. You can set the `TOCK_ARCH` to any architecture GCC's
`-mcpu` option accepts. Boards set an appropriate architecture by default,
(e.g. `cortex-m4` for the `storm` board).

To compile an app, `cd` to the desired app and `make`. For example:

```bash
$ cd userland/examples/blink/
$ make
```

This will build the app and generate a binary in Tock Binary Format
(using the `elf2tbf` utility):
`userland/examples/blink/build/cortex-m4/app.bin`.

Alternatively, apps can be built and automatically uploaded from the
Tock root directory:

```bash
$ make examples/blink
```

## Loading the kernel and applications onto a board.

This is generally done with `make program` and `make flash`, but is board
specific. To learn how to program your specific hardware, please see
the board specific READMEs:

* [imix](../boards/imix/README.md)
* [Hail](../boards/hail/README.md)
* [nRF](../boards/nrf51dk/README.md)
* [Storm](../boards/storm/README.md)


## Formatting Rust Source Code

Rust includes a tool for automatically formatting Rust source
code. This requires a `cargo` tool:

    $ cargo install rustfmt

Then run:

    $ make format

to format the repository.
