// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

pub mod helpers;
pub mod qemu;

use crate::test_cases::Requirement;
use anyhow::{Result, anyhow};
use std::path::Path;
use std::time::Duration;

/// Collection of boards supported by this end-to-end harness.
///
/// Implemented through a [`distributed_slice`](linkme::distributed_slice),
/// such that this slice gets assembled at link-time and we don't need to have a
/// single registry for it.
#[linkme::distributed_slice]
pub static BOARDS: [Board];

/// Description of a board supported by this harness.
///
/// Each board is registered in [`BOARDS`]. Boards are compared by their
/// [`name`](Board::name).
#[derive(Debug)]
pub struct Board {
    /// Name of the board in plans, test ids and on the command line. This name
    /// must be unique among [`BOARDS`].
    pub name: &'static str,
    /// Tock board directory in which the kernel is built, relative to the Tock
    /// repository.
    pub dir: &'static str,
    /// Path of the kernel binary produced by the build, relative to the Tock
    /// repository.
    pub kernel: &'static str,
    /// libtock-c `TOCK_TARGETS` to build apps for.
    ///
    /// Apps are only built for specific boards, which accelerates builds and
    /// avoids us having to depend on compilers for architecture we don't intend
    /// to execute on.
    ///
    /// Entries can reference variables from libtock-c's `Configuration.mk` as
    /// those get expanded by the call to `make`.
    pub tock_targets: &'static [&'static str],
    /// Upper bounds on the duration of certain board-specific operations.
    pub timing: Timing,
    /// Open the board for running tests, with the Tock repository at the given
    /// path.
    pub open: fn(&Path) -> Box<dyn Dut>,
}

/// Upper bounds on the duration of a board's operations.
///
/// A test's worst-case duration is computed from these bounds.
#[derive(Debug)]
pub struct Timing {
    /// Maximum time to write the kernel and apps to the board and start them.
    ///
    /// Exceeding this time will abort tests and report an error.
    pub flash: Duration,
    /// Maximum time for the board to boot after flashing.
    ///
    /// This time is added to the timeout of the test's first wait, or following
    /// every board reset.
    pub boot: Duration,
    /// Maximum time a test waits for expected console output.
    pub expect: Duration,
}

impl Board {
    /// Look up a board in [`BOARDS`] by name.
    ///
    /// Returns an error which lists all boards if no board has this name.
    pub fn from_name(name: &str) -> Result<&'static Board> {
        BOARDS.iter().find(|b| b.name == name).ok_or_else(|| {
            let mut names: Vec<_> = BOARDS.iter().map(|b| b.name).collect();
            names.sort();
            anyhow!(
                "unknown board {name:?}, expected one of: {}",
                names.join(", ")
            )
        })
    }

    /// Check if the board provides `requirement`.
    pub fn provides(&self, requirement: Requirement) -> bool {
        match requirement {
            Requirement::Uart => true,
        }
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

/// Serialize and deserialize a [`Board`] by its name.
///
/// Module for use with `#[serde(with = ...)]` to serialize a Board to its name,
/// and to deserialize a name of a board by trying to find a match in
/// [`BOARDS`](super::BOARDS).
pub mod by_name {
    use super::Board;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::borrow::Cow;

    pub fn serialize<S: Serializer>(board: &&'static Board, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(board.name)
    }

    /// Deserialize a board name and look up the board in
    /// [`BOARDS`](super::BOARDS).
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<&'static Board, D::Error> {
        let name = Cow::<'de, str>::deserialize(d)?;
        Board::from_name(&name).map_err(serde::de::Error::custom)
    }
}

/// Console of a board.
///
/// Implementations provide [`Uart::receive_until`]. Callers use
/// [`Uart::wait_for`].
pub trait Uart {
    /// Wait up to `timeout` for `needle` on the console. This returns the
    /// output up to and including `needle`.
    fn receive_until(&mut self, needle: &str, timeout: Duration) -> Result<String>;
    /// Return all output received since the console was opened.
    fn transcript(&mut self) -> &[u8];

    /// Wait up to `timeout` for `needle` on the console. This behaves like
    /// [`Uart::receive_until`], and additionally logs the wait and the received
    /// output.
    fn wait_for(&mut self, needle: &str, timeout: Duration) -> Result<String> {
        log::info!("  waiting for {needle:?} (up to {timeout:?})");
        let out = self.receive_until(needle, timeout)?;
        log::debug!("  received {out:?}");
        Ok(out)
    }
}

/// Device under test.
///
/// Whereas a [`Board`] statically describes a board "kind", a `Dut` is a single
/// instance of that kind, opened through [`Board::open`], and able to run
/// tests.
///
/// It holds the state of this device, such as connections to an emulator or to
/// the device's console.
pub trait Dut {
    /// Return the board this device is an instance of.
    fn board(&self) -> &'static Board;
    /// Write `kernel` and `apps` to the device and start them.
    fn flash(&mut self, kernel: &Path, apps: &[&Path]) -> Result<()>;
    /// Stop the software running on the device. A subsequent [`Dut::flash`]
    /// starts it again.
    fn reset(&mut self) -> Result<()>;
    /// Return the device's console, or `None` if the device has no console.
    fn uart(&mut self) -> Option<&mut dyn Uart>;
}

#[cfg(test)]
mod tests {
    use super::{BOARDS, by_name};

    #[test]
    fn board_names_are_unique() {
        let mut names: Vec<_> = BOARDS.iter().map(|b| b.name).collect();
        names.sort();
        let len = names.len();
        names.dedup();
        assert_eq!(names.len(), len, "a board name is used twice");
    }

    #[test]
    fn serialized_as_name() {
        for board in BOARDS {
            let json = by_name::serialize(&board, serde_json::value::Serializer).unwrap();
            assert_eq!(json, board.name);
            let back = by_name::deserialize(json).unwrap();
            assert!(std::ptr::eq(back, board));
        }
    }
}
