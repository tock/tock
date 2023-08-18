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
    + [Platform Build Scripts](#platform-build-scripts)
  * [LLVM Binutils](#llvm-binutils)
  * [Special `.apps` section](#special-apps-section)
- [Compiling a process](#compiling-a-process)
  * [Executing without Virtual Memory](#executing-without-virtual-memory)
    + [Position Independent Code](#position-independent-code)
    + [Fixed Address Loading](#fixed-address-loading)
      - [Fixed Address TBF Header](#fixed-address-tbf-header)
      - [Loading Fixed Address Processes into Flash](#loading-fixed-address-processes-into-flash)
      - [Booting Fixed Address Processes](#booting-fixed-address-processes)
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
    `SubSlice`, and the Hardware Interface Layer (HIL) definitions. This is
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

  * A platform-specific (e.g. _Imix_) crate that configures the chip and
    its peripherals, assigns peripherals to drivers, sets up virtualization
    layers, and defines a system call interface. This is located in `boards/`.

These crates are compiled using [Cargo](http://doc.crates.io), Rust's package
manager, with the platform crate as the base of the dependency graph. In
practice, the use of Cargo is masked by the Makefile system in Tock. Users can
simply type `make` from the proper directory in `boards/` to build the kernel
for that platform.

Internally, the Makefile is simply invoking Cargo to handle the build. For
example, `make` on the imix platform roughly translates to:

```bash
$ cargo build --release --target=thumbv7em-none-eabi
```

The `--release` argument tells Cargo to invoke the Rust compiler with
optimizations turned on. `--target` points Cargo to the target specification
which includes the LLVM data-layout definition and architecture definitions for
the compiler. Note, Tock uses additional compiler and linker flags to generate
correct and optimized kernel binaries for our supported embedded targets.


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

#### Platform Build Scripts

Cargo supports [build
scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) when
compiling crates, and each Tock platform crate includes a `build.rs` build
script. In Tock, these build scripts are primarily used to instruct cargo to
rebuild the kernel if a linker script changes.

Cargo's `build.rs` scripts are small Rust programs that must be compiled as part
of the kernel build process. Since these scripts execute on the host machine,
this means building Tock requires a Rust toolchain valid for the host machine
and its architecture. Cargo runs the compiled build script when compiling the
platform crate.

### LLVM Binutils

Tock uses the `lld`, `objcopy`, and `size` tools included with the Rust
toolchain to produce kernel binaries that are executed on microcontrollers. This
has two main ramifications:

1. The tools are not entirely feature-compatible with the GNU versions. While
   they are very similar, there are edge cases where they do not behave exactly
   the same. This will likely improve with time, but it is worth noting in case
   unexpected issues arise.
2. The tools will automatically update with Rust versions. The tools are
   provided in the `llvm-tools` rustup component that is compiled for and ships
   with every version of the Rust toolchain. Therefore, if Rust updates the
   version they use in the Rust repository, Tock will also see those updates.

### Special `.apps` section

Tock kernels include a `.apps` section in the kernel .elf file that is at the
same physical address where applications will be loaded. When compiling the
kernel, this is just a placeholder and is not populated with any meaningful
data. It exists to make it easy to update the kernel .elf file with an
application binary to make a monolithic .elf file so that the kernel and apps
can be flashed together.

When the Tock build system creates the kernel binary, it explicitly removes this
section so that the placeholder is not included in the kernel binary.

To use the special `.apps` section, `objcopy` can replace the placeholder with
an actual app binary. The general command looks like:

```bash
$ arm-none-eabi-objcopy --update-section .apps=libtock-c/examples/c_hello/build/cortex-m4/cortex-m4.tbf target/thumbv7em-none-eabi/release/stm32f412gdiscovery.elf target/thumbv7em-none-eabi/release/stm32f4discovery-app.elf
```

This replaces the placeholder section `.apps` with the "c_hello" application TBF
in the stm32f412gdiscovery.elf kernel ELF, and creates a new .elf called
`stm32f4discovery-app.elf`.

## Compiling a process

Unlike many other embedded systems, compilation of application code is entirely
separated from the kernel in Tock. An application uses a `libtock` library and
is built into a free-standing binary. The binary can then be uploaded onto a
Tock platform with an already existing kernel to be loaded and run.

Tock can support applications using any programming language and compiler
provided the applications can run with only access to fixed regions in flash and
RAM and without virtual memory.

Each Tock process requires a header that informs the kernel of the size of the
application's binary and where the location of the entry point is within the
compiled binary.

### Executing without Virtual Memory

Tock supports resource constrained microcontrollers which do not support virtual
memory. This means Tock process cannot assume a known address space. Tock
supports two methods for enabling processes despite the lack of virtual memory:
embedded PIC (FDPIC) and fixed address loading.

#### Position Independent Code

Since Tock loads applications separately from the kernel and is capable of
running multiple applications concurrently, applications cannot know in advance
at which address they will be loaded. This problem is common to many computer
systems and is typically addressed by dynamically linking and loading code at
runtime.

Tock, however, makes a different choice and requires applications to be compiled
as position independent code. Compiling with FDPIC makes all control flow
relative to the current PC, rather than using jumps to specified absolute
addresses. All data accesses are relative to the start of the data segment for
that app, and the address of the data segment is stored in a register referred
to as the `base register`. This allows the segments in Flash and RAM to be
placed anywhere, and the OS only has to correctly initialize the base register.

FDPIC code can be inefficient on some architectures such as x86, but the ARM
instruction set is optimized for FDPIC operation and allows most code to execute
with little to no overhead. Using FDPIC still requires some fixup at runtime, but
the relocations are simple and cause only a one-time cost when an application is
loaded. A more in-depth discussion of dynamically loading applications can be
found on the Tock website: [Dynamic Code Loading on a
MCU](http://www.tockos.org/blog/2016/dynamic-loading/).

For applications compiled with `arm-none-eabi-gcc`, building FDPIC code for Tock
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

#### Fixed Address Loading

Unfortunately, not all compilers support FDPIC. As of August 2023, LLVM and
riscv-gcc both do not support FDPIC. This complicates running Tock processes,
but Tock supports an alternative method using fixed addresses. This method works
by compiling Tock processes for fixed addresses in both flash and RAM (as
typical embedded compilation would do) and then processes are placed in flash so
that they match their fixed flash address and the kernel sets their RAM region
so their RAM addresses match. While this simplifies compilation, ensuring that
those addresses are properly met involves several components.

##### Fixed Address TBF Header

The first step is the linker must communicate which addresses it expects the
process to be placed at in both flash and RAM at execution time. It does this
with two symbols in the `.elf` file:

- `_flash_origin`: The address in flash the app was compiled for.
- `_sram_origin`: The address in ram the app was compiled for.

These symbols are then parsed by `elf2tab`. `elf2tab` uses `_flash_origin` to
ensure the `.tbf` file is properly created so that the compiled binary will end
up at the correct address. Both `_flash_origin` and `_sram_origin` are used to
create a `FixedAddresses` TBF TLV that is included in the TBF header. An example
of the Fixed Addresses TLV:

```
TLV: Fixed Addresses (5)                        [0x40 ]
  fixed_address_ram   :  536920064   0x2000c000
  fixed_address_flash :  268599424   0x10028080
```

With the Fixed Addresses TLV included in the TBF header, the kernel and other
tools now understand that for this process its address requirements must be met.

By convention, userspace apps compiled for fixed flash and RAM addresses include
the addresses in the `.tbf` filenames. For example, the leds example compiled as
a libtock-rs app might have a TAB that looks like:

```
[STATUS ] Inspecting TABs...
TAB: leds
  build-date: 2023-08-08 22:24:07+00:00
  minimum-tock-kernel-version: 2.1
  tab-version: 1
  included architectures: cortex-m0, cortex-m4, riscv32imc
  tbfs:
   cortex-m0.0x10020000.0x20004000
   cortex-m0.0x10028000.0x2000c000
   cortex-m4.0x00030000.0x20008000
   cortex-m4.0x00038000.0x20010000
   cortex-m4.0x00040000.0x10002000
   cortex-m4.0x00040000.0x20008000
   cortex-m4.0x00042000.0x2000a000
   cortex-m4.0x00048000.0x1000a000
   cortex-m4.0x00048000.0x20010000
   cortex-m4.0x00080000.0x20006000
   cortex-m4.0x00088000.0x2000e000
   riscv32imc.0x403b0000.0x3fca2000
   riscv32imc.0x40440000.0x3fcaa000
```

##### Loading Fixed Address Processes into Flash

When installing fixed address processes on a board the loading tool must ensure
that it places the TBF at the correct address in flash so that the process
binary executes at the address the linker intended. Tockloader supports
installing apps on boards and placing them at their fixed address location.
Tockloader will try to find a sort order based on available TBFs to install all
of the requested apps at valid fixed addresses.

With the process loaded at its fixed flash address, its essential that the RAM
address the process is expecting can also be met. However, the valid RAM
addresses for process is determined by the memory the kernel has reserved for
processes. Typically, this memory region is dynamic based on memory the kernel
is not using. The loader tool needs to know what memory is available for
processes so it can choose the compiled TBF that expects a RAM address the
kernel will actually be able to satisfy.

For the loader tool to learn what RAM addresses are available for processes the
kernel includes a TLV kernel attributes structure in flash immediately before
the start of apps. Tockloader can read these attributes to determine the valid
RAM range for processes so it can choose suitable TBFs when installing apps.

##### Booting Fixed Address Processes

The final step is for the kernel to initialize and execute processes. The
processes are already stored in flash, but the kernel must allocate a RAM region
that meets the process's fixed RAM requirements. The kernel will leave gaps in
RAM between processes to ensure processes have the RAM addresses they expected
during compilation.


### Tock Binary Format

In order to be loaded correctly, applications must follow the [Tock Binary
Format](TockBinaryFormat.md). This means the initial bytes of a Tock app must
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

```bash
tar cf app.tab cortex-m0.bin cortex-m4.bin metadata.toml
```

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
