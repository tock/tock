// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use std::process::Command;

/// Construct an OpenOCD invocation which runs `commands` and exits.
///
/// The invocation selects the debug probe with `interface`, the probe with the
/// serial number `serial` if given, and the target with `target`.
pub fn command(
    interface: &[&str],
    serial: Option<&str>,
    target: &[&str],
    commands: &[&str],
) -> Command {
    let mut cmd = Command::new("openocd");

    cmd.args(interface);
    if let Some(serial) = serial {
        cmd.args(["-c", &format!("adapter serial {serial}")]);
    }
    cmd.args(target);

    for command in commands {
        cmd.args(["-c", command]);
    }
    cmd.args(["-c", "exit"]);

    cmd
}

/// Construct an OpenOCD command which loads the first of the config files
/// `paths` that OpenOCD finds in its search path.
///
/// Config files move between OpenOCD releases (e.g., from `target/nrf52.cfg` to
/// `target/nordic/nrf52.cfg`). Listing each location supports releases from
/// before and after such a move.
pub fn source_first(paths: &[&str]) -> String {
    let paths = paths.join(" ");
    format!(
        "set found 0; \
         foreach path {{{paths}}} {{ \
           if {{!$found && ![catch {{find $path}} cfg]}} {{ source $cfg; set found 1 }} \
         }}; \
         if {{!$found}} {{ error \"none of {paths} is in the search path\" }}"
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn source_first_loads_the_first_config_found() {
        assert_eq!(
            super::source_first(&["a.cfg", "b.cfg"]),
            "set found 0; foreach path {a.cfg b.cfg} { \
             if {!$found && ![catch {find $path} cfg]} { source $cfg; set found 1 } }; \
             if {!$found} { error \"none of a.cfg b.cfg is in the search path\" }"
        );
    }
}
