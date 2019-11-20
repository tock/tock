# How does Tock compile?

There are two types of compilation artifacts in Tock: the kernel and user-level
processes (i.e. apps). Each type compiles differently. In addition, each
platform has a different way of programming the kernel and processes. Below is
an explanation of both kernel and process compilation as well as some examples
of how platforms program each onto an actual board.

<!-- npm i -g markdown-toc; markdown-toc -i Compilation.md -->

<!-- toc -->

- [Compiling the kernel](#compiling-the-kernel)
  * [Life of a Tock compilation](#life-of-a-tock-compilation)
  * [LLVM Binutils](#llvm-binutils)
- [Compiling a process](#compiling-a-process)
  * [Position Independent Code](#position-independent-code)
  * [Tock Binary Format](#tock-binary-format)
  * [Tock Application Bundle](#tock-application-bundle)
    + [TAB Format](#tab-format)
    + [Metadata](#metadata)
- [Loading the kernel and processes onto a board](#loading-the-kernel-and-processes-onto-a-board)

<!-- tocstop -->

## Compiling the kernel

The kernel is divided into five Rust crates (i.e. packages):

  * A core kernel crate containing key kernel operations such as handling
    interrupts and scheduling processes, shared kernel libraries such as
    `TakeCell`, and the Hardware Interface Layer (HIL) definitions. This is
    located in the `kernel/` folder.

  * An architecture (e.g. _ARM Cortex M4_) crate that implements context
    switching, and provides memory protection and systick drivers. This is
    located in the `arch/` folder.

  * A chip-specific (e.g. _Atmel SAM4L_) crate which handles interrupts and
    implements the hardware abstraction layer for a chip's peripherals. This is
    located in the `chips/` folder.

  * One (or more) crates for hardware independent drivers and virtualization
    layers. This is the `capsules/` folder in Tock. External projects using
    Tock may create additional crates for their own drivers.

  * A platform specific (e.g. _Imix_) crate that configures the chip and
    its peripherals, assigns peripherals to drivers, sets up virtualization
    layers, and defines a system call interface. This is located in `boards/`.

These crates are compiled using [Cargo](http://doc.crates.io), Rust's package
manager, with the platform crate as the base of the dependency graph. In
practice, the use of Cargo is masked by the Makefile system in Tock. Users can
simply type `make` from the proper directory in `boards/` to build the kernel
for that platform.

Internally, the Makefile is simply invoking Cargo to handle the build. For
example, `make` on the imix platform translates to:

```bash
$ cargo build --release --target=thumbv7em-none-eabi
```

The `--release` argument tells Cargo to invoke the Rust compiler with
optimizations turned on. `--target` points Cargo to the target specification
which includes the LLVM data-layout definition and architecture definitions for
the compiler.


### Life of a Tock compilation

When Cargo begins compiling the platform crate, it first resolves all
dependencies recursively. It chooses package versions that satisfy the
requirements across the dependency graph. Dependencies are defined in each
crate's `Cargo.toml` file and refer to paths in the local file-system, a
remote git repository, or a package published on [crates.io](http://crates.io).

Second, Cargo compiles each crate in turn as dependencies are satisfied. Each
crate is compiled as an `rlib` (an `ar` archive containing object files)
and combined into an executable ELF file by the compilation of the platform
crate.

You can see each command executed by `cargo` by passing it the `--verbose`
argument. In our build system, you can run `make V=1` to see the verbose
commands.


### LLVM Binutils

Tock uses the `lld`, `objcopy`, and `size` tools included with the Rust
toolchain to produce kernel binaries that are executed on microcontrollers. This
has three main ramifications:

1. The tools are not entirely feature-compatible with the GNU versions. While
   they are very similar, there are edge cases where they do not behave exactly
   the same. This will likely improve with time, but it is worth noting in case
   unexpected issues arise.
2. The tools will automatically update with Rust versions. The tools are
   provided in the `llvm-tools` rustup component that is compiled for and ships
   with every version of the Rust toolchain. Therefore, if Rust updates the
   version they use in the Rust repository, Tock will also see those updates.
3. Tock no longer relies on an external dependency to provide these tools. That
   should ensure that all Tock developers are using the same version of the
   tools.

## Compiling a process

Unlike many other embedded systems, compilation of application code is entirely
separated from the kernel in Tock. An application is combined with at least two
libraries: `libtock` and `newlib` and built into a free-standing binary. The
binary can then be uploaded onto a Tock platform with an already existing
kernel to be loaded and run.

Tock can support any programming language and compiler provided they meet
the following requirements:

 1. The application must be built as position independent code (PIC).

 2. The application must be linked with a loader script that places Flash
    contents above address `0x80000000` and RAM contents below it.

 3. The application binary must start with a header detailing the location of
    sections in the binary.

The first requirement is explained directly below while the second two are
detailed in [Tock Binary Format](#tock-binary-format).


### Position Independent Code

Since Tock loads applications separately from the kernel and is capable of
running multiple applications concurrently, applications cannot know in advance
at which address they will be loaded. This problem is common to many computer
systems and is typically addressed by dynamically linking and loading code at
runtime.

Tock, however, makes a different choice and requires applications to be compiled
as position independent code. Compiling with PIC makes all control flow relative
to the current PC, rather than using jumps to specified absolute addresses. All
data accesses are relative to the start of the data segment for that app, and
the address of the data segment is stored in a register referred to as the `base
register`. This allows the segments in Flash and RAM to be placed anywhere, and
the OS only has to correctly initialize the base register.

PIC code can be inefficient on some architectures such as x86, but the ARM
instruction set is optimized for PIC operation and allows most code to execute
with little to no overhead. Using PIC still requires some fixup at runtime, but
the relocations are simple and cause only a one-time cost when an application
is loaded.  A more in-depth discussion of dynamically loading applications can
be found on the Tock website: [Dynamic Code Loading on a
MCU](http://www.tockos.org/blog/2016/dynamic-loading/).

For applications compiled with `arm-none-eabi-gcc`, building PIC code for Tock
requires four flags:

 - `-fPIC`: only emit code that uses relative addresses.
 - `-msingle-pic-base`: force the use of a consistent _base register_ for the
   data sections.
 - `-mpic-register=r9`: use register r9 as the base register.
 - `-mno-pic-data-is-text-relative`: do not assume that the data segment is
   placed at a constant offset from the text segment.

Each Tock application uses a linker script that places Flash at address
`0x80000000` and SRAM at address `0x00000000`. This allows relocations pointing
at Flash to be easily differentiated from relocations pointing at RAM.

### Tock Binary Format

In order to be loaded correctly, applications must follow the [Tock Binary
Format](TockBinaryFormat.md). This means the first bytes of a Tock app must
follow this format so that Tock can load the application correctly.

In practice, this is automatically handled for applications. As part of the
compilation process, a tool called [Elf to TAB](https://github.com/tock/elf2tab)
does the conversion from ELF to Tock's expected binary format, ensuring that
sections are placed in the expected order, adding a section that lists necessary
load-time relocations, and creating the TBF header.


### Tock Application Bundle

To support ease-of-use and distributable applications, Tock applications are
compiled for multiple architectures and bundled together into a "Tock
Application Bundle" or `.tab` file. This creates a standalone file for an
application that can be flashed onto any board that supports Tock, and removes
the need for the board to be specified when the application is compiled.
The TAB has enough information to be flashed on many or all Tock compatible
boards, and the correct binary is chosen when the application is flashed
and not when it is compiled.

#### TAB Format

`.tab` files are `tar`ed archives of TBF compatible binaries along with a
`metadata.toml` file that includes some extra information about the application.
A simplified example command that creates a `.tab` file is:

    tar cf app.tab cortex-m0.bin cortex-m4.bin metadata.toml

#### Metadata

The `metadata.toml` file in the `.tab` file is a TOML file that contains a
series of key-value pairs, one per line, that provides more detailed information
and can help when flashing the application. Existing fields:

```
tab-version = 1                         // TAB file format version
name = "<package name>"                 // Package name of the application
only-for-boards = <list of boards>      // Optional list of board kernels that this application supports
build-date = 2017-03-20T19:37:11Z       // When the application was compiled
```

## Loading the kernel and processes onto a board

There is no particular limitation on how code can be loaded onto a board. JTAG
and various bootloaders are all equally possible. For example, the `hail` and
`imix` platforms primarily use the serial "tock-bootloader", and the other
platforms use jlink or openocd to flash code over a JTAG connection. In general,
these methods are subject to change based on whatever is easiest for users of
the platform.

In order to support multiple concurrent applications, the easiest option is to
use `tockloader` ([git repo](https://github.com/tock/tockloader)) to
manage multiple applications on a platform. Importantly, while applications
currently share the same upload process as the kernel, they are planned to
support additional methods in the future. Application loading through wireless
methods especially is targeted for future editions of Tock.
