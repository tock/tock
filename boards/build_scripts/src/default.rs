// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Provide helpers for building Tock boards that match the default conventions.

use std::fs;
use std::path::Path;

const LINKER_SCRIPT: &str = "layout.ld";

/// Setup the Tock board to build with a board-provided linker script called
/// `layout.ld`.
///
/// The board linker script (i.e., `layout.ld`) should end with the command:
///
/// ```
/// INCLUDE tock_kernel_layout.ld
/// ```
///
/// This function will ensure that the linker's search path is configured to
/// find `tock_kernel_layout.ld`.
pub fn default_linker_script() {
    if !Path::new(LINKER_SCRIPT).exists() {
        panic!("Boards must provide a `layout.ld` link script file");
    }

    rustflags_check();

    include_tock_kernel_layout();

    add_board_dir_to_linker_search_path();

    set_and_track_linker_script(LINKER_SCRIPT);
}

/// Include the folder where the board's Cargo.toml is in the linker file
/// search path.
pub fn add_board_dir_to_linker_search_path() {
    // Note this is a different path than the one returned by
    // `std::env!("CARGO_MANIFEST_DIR")` in `include_tock_kernel_layout()`,
    // since that is evaluated at compile
    // time while this `std::env::var("CARGO_MANIFEST_DIR")` is evaluated at runtime.
    println!(
        "cargo:rustc-link-arg=-L{}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    );
}

/// Include the folder where this build_script crate's Cargo.toml is in the
/// linker file search path for `tock_kernel_layout.ld`, and instruct cargo
/// to rebuild if that linker script is changed.
pub fn include_tock_kernel_layout() {
    println!("cargo:rustc-link-arg=-L{}", std::env!("CARGO_MANIFEST_DIR"));
    // Directive to rebuild if the linker script in this crate is changed.
    println!(
        "cargo:rerun-if-changed={}",
        Path::new(std::env!("CARGO_MANIFEST_DIR"))
            .join("tock_kernel_layout.ld")
            .to_string_lossy()
    );
}

pub fn rustflags_check() {
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
                "Incorrect build configuration. Verify you are using unstable cargo and have not unintentionally set the RUSTFLAGS environment variable."
            );
        }
    }
}

/// Pass the given linker script to cargo, and track it and all of its `INCLUDE`s
pub fn set_and_track_linker_script<P: AsRef<Path> + ToString>(path: P) {
    // Use the passed linker script
    println!("cargo:rustc-link-arg=-T{}", path.to_string());
    track_linker_script(path);
}

/// Track the given linker script and all of its `INCLUDE`s so that the build
/// is rerun when any of them change.
pub fn track_linker_script<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();

    // Skip the default Tock linker script as we have manually added the
    // containing directory to the linker search path and we do not know the
    // path to add the rerun directive here. Instead, we add the rerun directory
    // for the default Tock linker script manually before calling this function.
    if path.to_str() == Some("tock_kernel_layout.ld") {
        return;
    }

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
