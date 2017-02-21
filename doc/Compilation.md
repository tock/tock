# How does Tock compile?

There are two types of compilation artifacts in Tock: the kernel and user-level
processes (i.e. apps). Each type compiles differently. In addition, each
platform has a different way of programming the kernel and processes. Below is
an explanation of both kernel and process compilation as well as some examples
of how platforms program each onto an actual board.

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
example, `make` on the Imix platform translates to:

```bash
$ cargo build --release --target=sam4l.json
```

The `--release` argument tells Cargo to invoke the Rust compiler with
optimizations turned on and without debug symbols. `--target` points Cargo to
the target specification which includes the LLVM data-layout definition,
architecture definitions for the compiler, arguments to pass to the linker and
compilation options such as floating-point support.


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
argument.


## Compiling a process

Unlike many other embedded systems, compilation of application code is entirely
separated from the kernel in Tock. An application is combined with two
libraries: `libtock` and `newlib` and built into a free-standing binary. The
binary can then be uploaded onto a Tock platform with an already existing
kernel to be loaded and run. For more details about application code, see
[Userland](./Userland.md).

Currently, all Tock platforms are ARM Cortex-M processors and all existing
applications are written in C. Therefore, compilation uses `arm-none-eabi-gcc`.
Alternative languages and compilers are all possible for building applications,
as long as they build code following several requirements:

 1) The application must be built as position independent code (PIC)

 2) The application must be linked with a loader script that places Flash
    contents above address `0x80000000` and RAM contents below it

 3) The application binary must start with a header detailing the location of
    sections in the binary

The first requirement is explained directly below while the second two are
detailed in [Tock Binary Format](#tock-binary-format). Again as with the
kernel, the compilation process is handled by Makefiles and the user does not
normally need to interact with it.


### Position Independent Code

Since Tock loads applications separately from the kernel and is capable of
running multiple applications concurrently, applications cannot know in advance
at which address they will be loaded. This problem is common to many computer
systems and is typically addressed by dynamically linking and loading code at
runtime.

In Tock, however, we make a different choice and require applications to be
compiled as position independent code. Compiling with PIC makes all control
flow relative to the current PC, rather than using jumps to specified absolute
addresses. All data accesses are relative to the start of the data segment for
that app, and the address of the data segment is stored in a register referred
to as the `base register`. This potentially allows the segments in Flash and
RAM to be placed anywhere, and as long as the OS correctly initializes the base
register, everything will work fine.

PIC code can be inefficient on some architectures such as x86, but the ARM
instruction set is optimized for PIC operation and allows most code to execute
with little to no overhead. Using PIC still requires some fixup at runtime, but
the relocations are simple and cause only a one-time cost when an application
is loaded.  A more in-depth discussion of dynamically loading applications can
be found on the Tock website: [Dynamic Code Loading on a
MCU](http://www.tockos.org/blog/2016/dynamic-loading/).

For applications compiled with `arm-none-eabi-gcc`, building PIC code for Tock
requires four flags:

 - `-fPIC` - only emit code that uses relative addresses.
 - `-msingle-pic-base` - force the use of a consistent _base register_ for the
   data sections
 - `-mpic-register=r9` - use register r9 as the base register
 - `-mno-pic-data-is-text-relative` - do not assume that the data segment is
   placed at a constant offset from the text segment


### Tock Binary Format

In order to be loaded correctly, applications must follow the Tock Binary
Format. This means the use of a linker script following specific rules and a
header for the binary so that Tock can load the application correctly.

Each Tock application uses a
[linker script](https://github.com/helena-project/tock/blob/11871e5abcd5baf7c16ec951ac1fadd515851ec6/userland/linker.ld)
that places Flash at address `0x80000000` and SRAM at address `0x00000000`.
This allows relocations pointing at Flash to be easily differentiated from
relocations pointing at RAM.

Each Tock application begins with a header that is today defined as:

```rust
struct LoadInfo {
    version: u32,            // Version of the Tock Binary Format (currently 1)
    total_size: u32,         // Total padded size of the program image in bytes
    entry_offset: u32,       // The function to call to start the application
    rel_data_offset: u32,    // Offset in memory to start of relocation data
    rel_data_size: u32,      // Length of relocation data segment in bytes
    text_offset: u32,        // Offset in memory to start of text segment
    text_size: u32,          // Length of text segment in bytes
    got_offset: u32,         // Offset in memory to start of GOT
    got_size: u32,           // Length of GOT segment in bytes
    data_offset: u32,        // Offset in memory to start of data
    data_size: u32,          // Length of data segment in bytes
    bss_mem_offset: u32,     // Offset in memory to start of BSS
    bss_size: u32,           // Length of BSS segment in bytes
    min_stack_len: u32,      // Minimum stack size
    min_app_heap_len: u32    // Minimum size for the application heap
    min_kernel_heap_len: u32 // Minimum size for kernel's borrow heap
    pkg_name_offset: u32,    // Offset in memory to a string with package name
    pkg_name_size: u32,      // Length of package name in bytes
    checksum: u32,           // XOR of all previous fields
}
```

In practice, this is automatically handled for applications. As part of the
compilation process, a tool called
[Elf to Tock Binary Format](https://github.com/helena-project/tock/blob/a0a3b7705354db0e7dcfddd4063c7d6ec38be7a8/userland/tools/elf2tbf/src/main.rs)
does the conversion from ELF to Tock's expected binary format, ensuring that
sections are placed in the expected order, adding a section that lists
necessary load-time relocations, and creating the `LoadInfo` header.


### Note for the Future

All these requirements exist in current Tock, but are not fundamental. Future
version of Tock may support dynamic runtime application linking and loading.


## Loading the kernel and processes onto a board

There is no particular limitation on how code can be loaded onto a board. JTAG
and various bootloaders are all equally possible. Currently, the `storm`
platform uses either JTAG or a serial bootloader, the `imix` platform supports
JTAG, and the `nrf51dk` platform supports the mbed bootloader which presents
itself as a USB storage device that `.bin` files can be copied into. All of
these methods are subject to change based on whatever is easiest for users of
the platform.

In order to support multiple concurrent applications, the easiest option is to
use a script in [userland/tools/program/](../userland/tools/program/) to
combine multiple application binaries into a single image to be loaded.
Importantly, while applications currently share the same upload process as the
kernel, they are planned to support additional methods in the future.
Application loading through wireless methods especially is targeted for future
editions of Tock.


