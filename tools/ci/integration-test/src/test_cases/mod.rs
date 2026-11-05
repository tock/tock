// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

mod step;

pub use step::{Step, TestOutcome, run};

use crate::boards::{BOARDS, Board};

/// Collection of tests run by this end-to-end harness.
///
/// Implemented through a [`distributed_slice`](linkme::distributed_slice), such
/// that each test module registers its own test and we don't need a central
/// registry.
#[linkme::distributed_slice]
pub static TESTS: [TestCase];

mod hello_world;

/// Resource a test requires a board to provide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    Uart,
}

/// A test, which is planned on every board that provides all of its
/// requirements.
pub struct TestCase {
    /// Unique name of the test.
    pub id: &'static str,
    /// Requirements on the DUTs that this test case can operate on.
    pub requires: &'static [Requirement],
    /// libtock-c apps to install, as paths under libtock-c's `examples/`.
    pub apps: &'static [&'static str],
    /// Steps of the test, which are executed in order.
    pub steps: &'static [Step],
}

impl TestCase {
    /// Look up a test in [`TESTS`] by its id.
    ///
    /// Returns an error if no test with this id exists.
    pub fn from_id(id: &str) -> anyhow::Result<&'static TestCase> {
        (TESTS.iter())
            .find(|t| t.id == id)
            .ok_or_else(|| anyhow::anyhow!("unknown test {id:?}"))
    }

    /// Return the boards which provide all requirements of this test.
    pub fn boards(&self) -> impl Iterator<Item = &'static Board> + '_ {
        (BOARDS.iter()).filter(|board| self.requires.iter().all(|r| board.provides(*r)))
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
    use super::{BOARDS, Requirement, Step, TESTS};
    use crate::boards::qemu;

    #[test]
    fn every_test_file_is_registered() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/test_cases");
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_stem().unwrap().to_str().unwrap();
            if !["mod", "step"].contains(&name) {
                assert!(
                    TESTS.iter().any(|t| t.id == name),
                    "{name}.rs registers no test with id {name:?}"
                );
            }
        }
    }

    #[test]
    fn boards_follow_from_requirements() {
        let boards = |id: &str| -> Vec<_> {
            let tc = TESTS.iter().find(|t| t.id == id).unwrap();
            tc.boards().collect()
        };
        assert_eq!(boards("hello_world").len(), BOARDS.len());
    }

    #[test]
    fn test_ids_are_unique() {
        let mut ids: Vec<_> = TESTS.iter().map(|t| t.id).collect();
        ids.sort();
        let len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), len, "a test id is used twice");
    }

    #[test]
    fn console_steps_require_uart() {
        fn uses_console(step: &Step) -> bool {
            match *step {
                Step::Expect(_) => true,
            }
        }
        for tc in TESTS {
            if tc.steps.iter().any(uses_console) {
                assert!(tc.requires.contains(&Requirement::Uart), "{}", tc.id);
            }
        }
    }

    #[test]
    fn worst_cases_include_flashing_and_booting() {
        let hello = TESTS.iter().find(|t| t.id == "hello_world").unwrap();
        let timing = &qemu::RV64_VIRT.timing;
        assert_eq!(
            hello.worst_case(&qemu::RV64_VIRT),
            timing.flash + timing.boot + timing.expect
        );
    }
}
