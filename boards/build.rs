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

use std::env;
use std::fs;
use std::path::Path;

const LINKER_SCRIPT: &str = "layout.ld";

fn main() {
    assert!(
        Path::new(LINKER_SCRIPT).exists(),
        "Boards must provide a `{LINKER_SCRIPT}` linker script file"
    );

    // Provide the linker script, ensuring the build reruns when it changes.
    track_linker_script(LINKER_SCRIPT);
    println!("cargo:rustc-link-arg=-T{LINKER_SCRIPT}");

    // Allow the linker script to use paths relative to the board crate's root
    // (the directory that contains `Cargo.toml`).
    //
    // The following flag should only be passed to the board's binary crate, but
    // not to any of its dependencies (the kernel, capsules, chips, etc.). The
    // dependencies wouldn't use it, but because the link path is different for
    // each board, Cargo wouldn't be able to cache builds of the dependencies.
    //
    // Indeed, as far as Cargo is concerned, building the kernel with `-C
    // link-arg=-L/tock/boards/imix` is different than building the kernel with
    // `-C link-arg=-L/tock/boards/hail`, so Cargo would have to rebuild the
    // kernel for each board instead of caching it per board (even if in reality
    // the same kernel is built because the link-arg isn't used by the kernel).
    let cwd = env::current_dir().expect("failed to read current directory");
    println!("cargo:rustc-link-arg-bins=-L{}", cwd.display());
}

/// Track the given linker script and all of its `INCLUDE`s so that the build
/// is rerun when any of them change.
fn track_linker_script<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();

    assert!(path.is_file(), "expected path {path:?} to be a file");

    println!("cargo:rerun-if-changed={}", path.display());

    // Find all the `INCLUDE <relative path>` lines in the linker script.
    let link_script = fs::read_to_string(path).expect("failed to read {path:?}");
    let includes = link_script
        .lines()
        .filter_map(|line| line.strip_prefix("INCLUDE").map(str::trim));

    // Recursively track included linker scripts.
    for include in includes {
        track_linker_script(include);
    }
}
