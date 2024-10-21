// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! This build script can be used by Tock board crates to ensure that they are
//! rebuilt when there are any changes to the `layout.ld` linker script or any
//! of its `INCLUDE`s.
//!
//! Board crates can use this script from their `Cargo.toml` files:
//!
//! ```toml
//! [package]
//! # ...
//! build = "../path/to/build.rs"
//! ```
//!
//! Out-of-tree boards are recommended to copy this file into their board crate.

fn main() {
    tock_build_scripts::default_linker_script();
}
