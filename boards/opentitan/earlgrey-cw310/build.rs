// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! This board uses a custom build script to enable selecting a different layout
//! file for tests, which require a different layout than normal kernel builds.
//! The script is lightly adapted from the `default_linker_script` in
//! `tock_build_scripts`, and uses the functions provided by that crate.

use std::path::Path;

const LINKER_SCRIPT_OVERRIDE_ENV: &str = "LINKER_SCRIPT_OVERRIDE";

fn main() {
    let linker_script =
        std::env::var(LINKER_SCRIPT_OVERRIDE_ENV).unwrap_or("layout.ld".to_string());
    println!("cargo:rerun-if-env-changed={}", LINKER_SCRIPT_OVERRIDE_ENV);

    if !Path::new(&linker_script).exists() {
        panic!(
            "Boards must provide a linker script file; path does not exist: {:?}",
            linker_script
        );
    }
    tock_build_scripts::default::rustflags_check();
    tock_build_scripts::default::include_tock_kernel_layout();
    tock_build_scripts::default::add_board_dir_to_linker_search_path();
    tock_build_scripts::default::set_and_track_linker_script(linker_script);
}
