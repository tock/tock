// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

fn main() {
    tock_build_scripts::default::rustflags_check();
    tock_build_scripts::default::include_tock_kernel_layout();
    tock_build_scripts::default::add_board_dir_to_linker_search_path();

    // Single layout.  All boot paths land the image at 0x34200000.
    tock_build_scripts::default::set_and_track_linker_script("layout.ld");

    // Emit a linker map next to the elf so downstream build scripts
    // can copy it.  rust-lld interprets `-Map=<path>` relative to its CWD, which cargo
    // sets to CARGO_MANIFEST_DIR (this crate's directory).  Compute an
    // absolute path so the map lands in target/<triple>/release/ where
    // every consumer expects it, regardless of CARGO_TARGET_DIR.
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        format!("{manifest_dir}/../../target")
    });
    let target = std::env::var("TARGET").expect("TARGET not set");
    let profile = std::env::var("PROFILE").expect("PROFILE not set");
    let pkg_name = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME not set");
    let map_path = format!("{target_dir}/{target}/{profile}/{pkg_name}.map");
    println!("cargo:rustc-link-arg=-Map={map_path}");
}
