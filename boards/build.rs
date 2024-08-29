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

    // The `RUSTFLAGS` that the Tock config files set can be easily overridden
    // by command line flags. The build will still succeed but the resulting
    // binary may be invalid as it was not built with the intended flags. This
    // check seeks to prevent that. Our approach is we set a sentinel flag in
    // our configuration file and then check that it is set here. If it isn't,
    // the flags were overwritten and the build will be invalid.
    //
    // We only do this check if we are actually building for an embedded target
    // (i.e., the TARGET is not the same as the HOST). This avoids a false
    // positive when running tools like `cargo clippy`.
    //
    // If you are intentionally not using the standard Tock config files, set
    // `cfg-tock-buildflagssentinel` in your cargo config to prevent this
    // error.
    if std::env::var("HOST") != std::env::var("TARGET") {
        let rust_flags = std::env::var("CARGO_ENCODED_RUSTFLAGS");
        if !rust_flags
            .iter()
            .any(|f| f.contains("cfg_tock_buildflagssentinel"))
        {
            panic!(
                "Incorrect build configuration. \
            Verify you have not unintentionally set the RUSTFLAGS environment variable."
            );
        }
    }

    // Include the folder where the board's Cargo.toml is in the linker file
    // search path.
    println!("cargo:rustc-link-arg=-L{}", std::env!("CARGO_MANIFEST_DIR"));
    // `-Tlayout.ld`: Use the linker script `layout.ld` all boards must provide.
    println!("cargo:rustc-link-arg=-T{}", LINKER_SCRIPT);

    track_linker_script(LINKER_SCRIPT);
}

/// Track the given linker script and all of its `INCLUDE`s so that the build
/// is rerun when any of them change.
fn track_linker_script<P: AsRef<Path>>(path: P) {
    track_linker_script_inner(
        path.as_ref().to_path_buf(),
        std::env::current_dir().unwrap(),
    )
}
fn track_linker_script_inner(linker_script: std::path::PathBuf, directory: std::path::PathBuf) {
    let path = std::path::absolute(directory.join(linker_script)).unwrap();
    let parent_buf = path.parent().unwrap().to_path_buf();

    assert!(path.is_file(), "expected path {path:?} to be a file");

    println!("cargo:rerun-if-changed={}", path.display());

    // Find all the `INCLUDE <relative path>` lines in the linker script.
    let link_script = fs::read_to_string(path).expect("failed to read {path:?}");
    let includes = link_script
        .lines()
        .filter_map(|line| line.strip_prefix("INCLUDE").map(str::trim));

    // Recursively track included linker scripts.
    for include in includes {
        track_linker_script_inner(include.into(), parent_buf.clone());
    }
}
