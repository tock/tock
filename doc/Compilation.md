% How does Tock compile?

There are two types of compilation artifacts in Tock: the kernel and user-level
processes (i.e. apps). Each type compiles differently. In addition, each
platform has a different way of programming the kernel and processes. Below is
an explanation of both kernel and process compilation as well as some examples
of how platforms program each onto an actual board.

# Compiling the kernel

The kernel is divided into five Rust crates (i.e. packages):

  * `main` contains the scheduler, definitions for the the process type and
    traits for drivers, platforms and chips.

  * An architecture (e.g. _ARM Cortex M4_) crate that implements context
    switching, and provides memory protection and systick drivers.

  * A chip-specific (e.g. _Atmel SAM4L_) crate which handles interrupts and
    implements the hardware abstraction layer for a chip's peripherals.

  * One (or more) crates for hardware independnt drivers and virtualization 
    layers.

  * A platform specific (e.g. _Firestorm_) crate that configures the chip and
    its peripherals, assigns perpiherals to drivers, sets up virtualization
    layers and defines a system call interface.

These crates are compiled using [Cargo](http://doc.crates.io), Rust's package
manager, with the platform crate as the base of the dependency graph.

To compile the kernel, go to your platform's base directory and run:

```bash
$ cargo build --release --target=target.json
```

The `--release` argument tells cargo to invoke the Rust compiler with
optimizations turned on and without debug symbols. `--target` points cargo to
the target specification which includes the an LLVM data-layout definition,
architecture definitions for the compiler, arguments to pass to the linker and
compilation options such as floating-point support.

Platforms generally include a `Makefile` that invokes this command by default,
so you can just type:

```bash
$ make
```

## Life of a Tock compilation

When cargo begins compiling the platform crate, it first resolves all
dependencies recursively. It choosing package versions that satisfy the
requirements across the dependency graph. Dependencies are defined in each
crate's `Cargo.toml` file and refer to paths in the local file-system, a 
remote git repository or a package published on [crates.io](http://crates.io).

Second, Cargo compiles each crate in turn as dependencies are satisfied. Each 
crate is compiled as an `rlib` (an `ar` archive containing object files) 
and combined into an executable ELF file by the compilation of the platform 
crate.

You can see each command executed by `cargo` by passing the `--verbose`
argument.

# Compiling a process

TODO

# Loading the kernel and processes onto a board

TODO

