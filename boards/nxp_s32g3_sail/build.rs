// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

fn main() {
    tock_build_scripts::default::rustflags_check();
    tock_build_scripts::default::include_tock_kernel_layout();
    tock_build_scripts::default::add_board_dir_to_linker_search_path();

    let linker_script = if std::env::var_os("CARGO_FEATURE_GW_CORE0_INIT").is_some() {
        "layout_gw_core0.ld"
    } else {
        "layout.ld"
    };
    tock_build_scripts::default::set_and_track_linker_script(linker_script);

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR unset");
    let map = std::path::Path::new(&manifest_dir)
        .join("..")
        .join("..")
        .join("target")
        .join("thumbv7em-none-eabihf")
        .join("release")
        .join("nxp_s32g3_sail.map");

    println!("cargo:rustc-link-arg=-Map={}", map.display());
}
