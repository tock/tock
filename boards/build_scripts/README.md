Tock Build Scripts Support Crate
================================

This crate provides helpers for building Tock boards.

Out-of-tree Tock kernel configurations (i.e., Tock boards) can optionally
include this crate as a dependency to use the
[build.rs](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
functionality provided by the kernel.

This crate also packages the default `tock_kernel_layout.ld` linker script.

Usage
-----

There are three general steps to use this crate.

1. This crate should be included as a build dependency in the board's
   Cargo.toml file:

   ```toml
   # Cargo.toml

   # ...Existing Cargo.toml contents...

   [build-dependencies]
   tock_build_scripts = { git = "https://github.com/tock/tock"}
   ```

   This will ensure the crate and the build scripts are available for the board
   build.

2. This crate provides a helper function which can used from the board's
   build.rs file. In the common case, you just call the provided function from
   the build.rs file in your crate's root:

   ```rs
   // build.rs

   fn main() {
       tock_build_scripts::default_linker_script();
   }
   ```
