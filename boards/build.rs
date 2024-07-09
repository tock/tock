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

use std::fs;
use std::path::Path;

const LINKER_SCRIPT: &str = "layout.ld";

fn main() {
    if !Path::new(LINKER_SCRIPT).exists() {
        panic!("Boards must provide a `layout.ld` link script file");
    }

    println!("cargo:rustc-link-arg=-L{}", std::env!("CARGO_MANIFEST_DIR"));
    // `-Tlayout.ld`: Use the linker script `layout.ld` all boards must provide.
    println!("cargo:rustc-link-arg=-T{}", LINKER_SCRIPT);

    track_linker_script(LINKER_SCRIPT);
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
