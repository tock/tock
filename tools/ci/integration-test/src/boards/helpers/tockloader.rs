// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use super::command;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Write `kernel` at `address`, and `apps`, to the tockloader flash file
/// `flash_file` of the board `board` (as named by tockloader).
///
/// The apps are written at `app_address` if given, and at the default app
/// address of the board otherwise.
///
/// This will overwrite any existing kernel image (or other data) at `address`
/// and then modify the app list in the flash file (adding to the list if there
/// are already apps installed).
pub fn write_flash_file(
    flash_file: &Path,
    board: &str,
    address: u64,
    app_address: Option<u64>,
    kernel: &Path,
    apps: &[&Path],
) -> Result<()> {
    command::run(
        Command::new("tockloader")
            .arg("flash")
            .arg("--flash-file")
            .arg(flash_file)
            .args(["--board", board, "--address", &format!("{address:#x}")])
            .arg(kernel),
    )?;
    install_apps(flash_file, board, app_address, apps)
}

/// Install `apps` into the tockloader flash file `flash_file` of the board
/// `board` (as named by tockloader).
///
/// This will modify the app list in the flash file (adding to the list if there
/// are already apps installed).
pub fn install_apps(
    flash_file: &Path,
    board: &str,
    app_address: Option<u64>,
    apps: &[&Path],
) -> Result<()> {
    if apps.is_empty() {
        return Ok(());
    }
    let mut install = Command::new("tockloader");
    install
        .arg("install")
        .arg("--flash-file")
        .arg(flash_file)
        .args(["--board", board]);
    if let Some(app_address) = app_address {
        install.args(["--app-address", &format!("{app_address:#x}")]);
    }
    command::run(install.args(apps))
}
