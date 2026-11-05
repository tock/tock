// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

mod step;

// Test definitions in modules:
mod hello_world;

pub use step::{Step, TestOutcome, run};

use crate::boards::{BOARDS, Board};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Collection of tests run by this end-to-end harness.
#[linkme::distributed_slice]
pub static TESTS: [TestCase];

/// Tests of [`TESTS`] by their id.
///
/// Panics on first use if two tests share an id.
static TESTS_BY_ID: LazyLock<HashMap<&'static str, &'static TestCase>> = LazyLock::new(|| {
    let mut tests = HashMap::new();
    for test in TESTS {
        if tests.insert(test.id, test).is_some() {
            panic!("test id {:?} is used twice", test.id);
        }
    }
    tests
});

/// Resource a test requires a board to provide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    Uart,
}

/// App to install for a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum App {
    /// libtock-c app (a path under libtock-c's `examples/` dir)
    LibtockCExample(&'static str),
}

/// A test, which is planned on every board that provides all of its
/// requirements.
pub struct TestCase {
    /// Unique name of the test.
    pub id: &'static str,
    /// Requirements on the DUTs that this test case can operate on.
    pub requires: &'static [Requirement],
    /// Apps to install.
    pub apps: &'static [App],
    /// Steps of the test, which are executed in order.
    pub steps: &'static [Step],
}

impl TestCase {
    /// Look up a test in [`TESTS`] by its id.
    ///
    /// Returns an error if no test with this id exists.
    pub fn from_id(id: &str) -> anyhow::Result<&'static TestCase> {
        TESTS_BY_ID
            .get(id)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("unknown test {id:?}"))
    }

    /// Return the boards which provide all requirements of this test.
    pub fn boards(&self) -> impl Iterator<Item = &'static Board> + '_ {
        BOARDS
            .iter()
            .filter(|board| self.requires.iter().all(|r| board.provides(*r)))
    }

    /// Return the longest time the test may take on `board`. This includes
    /// flashing and booting the board.
    pub fn worst_case(&self, board: &Board) -> std::time::Duration {
        let timing = &board.timing;
        let steps: std::time::Duration = self.steps.iter().map(|s| s.cost(timing)).sum();
        timing.flash + timing.boot + steps
    }
}

/// Serialization of a `&'static TestCase` as its [`id`](TestCase::id), for use
/// with `#[serde(with = "crate::test_cases::by_id")]`.
pub mod by_id {
    use super::TestCase;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::borrow::Cow;

    pub fn serialize<S: Serializer>(test: &&'static TestCase, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(test.id)
    }

    /// Deserialize a test id and look up the test in [`TESTS`](super::TESTS).
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<&'static TestCase, D::Error> {
        let id = Cow::<'de, str>::deserialize(d)?;
        TestCase::from_id(&id).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{BOARDS, LazyLock, Requirement, Step, TESTS, TESTS_BY_ID, TestCase};
    use crate::boards::qemu_rv64_virt::QEMU_RV64_VIRT;

    /// A file in `test_cases/` which is not declared as a module is not
    /// compiled, and its tests would silently be missing from [`TESTS`].
    #[test]
    fn every_test_file_is_a_module() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/test_cases");
        let mod_rs = include_str!("mod.rs");
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_stem().unwrap().to_str().unwrap();
            if name != "mod" {
                assert!(
                    mod_rs.lines().any(|l| l.ends_with(&format!("mod {name};"))),
                    "{name}.rs is not declared as a module in test_cases/mod.rs"
                );
            }
        }
    }

    #[test]
    fn boards_follow_from_requirements() {
        let hello = TestCase::from_id("hello_world").unwrap();
        assert_eq!(hello.boards().count(), BOARDS.len());
    }

    #[test]
    fn test_ids_are_unique() {
        LazyLock::force(&TESTS_BY_ID);
    }

    #[test]
    fn console_steps_require_uart() {
        for tc in TESTS {
            if tc.steps.iter().any(|s| matches!(s, Step::Expect(_))) {
                assert!(tc.requires.contains(&Requirement::Uart), "{}", tc.id);
            }
        }
    }

    #[test]
    fn worst_cases_include_flashing_and_booting() {
        let hello = TestCase::from_id("hello_world").unwrap();
        let timing = &QEMU_RV64_VIRT.timing;
        assert_eq!(
            hello.worst_case(&QEMU_RV64_VIRT),
            timing.flash + timing.boot + timing.expect
        );
    }
}
