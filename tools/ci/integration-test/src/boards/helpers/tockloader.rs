// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use std::process::Command;

/// Construct a tockloader invocation with the subcommand `subcommand`.
pub fn command(subcommand: &str) -> Command {
    let mut cmd = Command::new("tockloader");
    cmd.arg(subcommand);
    cmd
}
