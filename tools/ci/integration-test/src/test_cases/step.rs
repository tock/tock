// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Test steps and their execution.
//!
//! A test consists of [`Step`]s, which are executed in order. The cost of a
//! step ([`Step::cost`]) is the longest time it may take on a board when all of
//! its waits time out. It is computed from the board's [`Timing`].
//!
//! Each step is bounded by its own cost, and there is no overall deadline. Time
//! spent on the host between steps is not accounted for.

use super::{Requirement, TestCase};
use crate::boards::{Dut, Timing, Uart};
use anyhow::{Result, anyhow, ensure};
use log::info;
use serde::Serialize;
use std::mem;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub enum Step {
    /// Wait for the given text on the console.
    Expect(&'static str),
}

impl Step {
    /// Return the longest time this step may take on a board with `timing`.
    pub fn cost(&self, timing: &Timing) -> Duration {
        match *self {
            Step::Expect(_) => timing.expect,
        }
    }
}

/// Return an error if the operation `what`, which started at `start`, took
/// longer than `bound`.
fn check_bound(what: &str, start: Instant, bound: Duration) -> Result<()> {
    let took = start.elapsed();
    ensure!(
        took <= bound,
        "{what} took {took:.1?}, longer than its {bound:?}"
    );
    Ok(())
}

/// Outcome of running a test.
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "status")]
pub enum TestOutcome {
    Passed,
    /// The test failed with `reason`.
    Failed {
        reason: String,
    },
    /// The board or the test infrastructure failed with `reason` (e.g., when
    /// flashing exceeded [`Timing::flash`]).
    InfraFailed {
        reason: String,
    },
    /// The test was skipped for `reason`.
    Skipped {
        reason: String,
    },
}

/// Flash `kernel` and `apps` onto `dut` and run the steps of `case`.
///
/// Flashing failures are reported as infrastructure failures.
pub fn run(dut: &mut dyn Dut, case: &TestCase, kernel: &Path, apps: &[&Path]) -> TestOutcome {
    let timing = &dut.board().timing;
    let start = Instant::now();

    info!("  flashing...");
    let flashed =
        (dut.flash(kernel, apps)).and_then(|()| check_bound("flashing", start, timing.flash));
    if let Err(e) = flashed {
        let reason = format!("{e:#}");
        return TestOutcome::InfraFailed { reason };
    }

    info!("  running test steps...");
    let mut ctx = Ctx {
        dut,
        requires: case.requires,
        timing,
        boot: timing.boot,
    };
    match case.steps.iter().try_for_each(|step| ctx.step(step)) {
        Ok(()) => TestOutcome::Passed,
        Err(e) => TestOutcome::Failed {
            reason: format!("{e:#}"),
        },
    }
}

struct Ctx<'a> {
    dut: &'a mut dyn Dut,
    requires: &'static [Requirement],
    timing: &'static Timing,
    /// Boot time which the test has not waited for yet. This time is added to
    /// the timeout of the next wait.
    boot: Duration,
}

impl Ctx<'_> {
    fn step(&mut self, step: &Step) -> Result<()> {
        match *step {
            Step::Expect(needle) => self.expect(needle).map(drop),
        }
    }

    fn uart(&mut self) -> Result<&mut dyn Uart> {
        ensure!(
            self.requires.contains(&Requirement::Uart),
            "test used the console without requiring `Uart`"
        );
        (self.dut.uart()).ok_or_else(|| anyhow!("board has no console"))
    }

    /// Wait for `needle` on the console. This returns the output up to and
    /// including `needle`.
    fn expect(&mut self, needle: &str) -> Result<String> {
        let timeout = self.timing.expect + mem::take(&mut self.boot);
        self.uart()?.wait_for(needle, timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::Step::*;
    use super::*;
    use crate::boards::helpers::console_buffer::ConsoleBuffer;
    use crate::boards::{Board, qemu};

    /// Mock board with `output` pending on its console.
    struct Console {
        output: ConsoleBuffer,
    }

    impl Uart for Console {
        fn receive_until(&mut self, needle: &str, _: Duration) -> Result<String> {
            (self.output.take_through(needle)).ok_or_else(|| anyhow!("no {needle:?}"))
        }
        fn transcript(&mut self) -> &[u8] {
            self.output.transcript()
        }
    }

    impl Dut for Console {
        fn board(&self) -> &'static Board {
            &qemu::RV64_VIRT
        }
        fn flash(&mut self, _: &Path, _: &[&Path]) -> Result<()> {
            Ok(())
        }
        fn reset(&mut self) -> Result<()> {
            Ok(())
        }
        fn uart(&mut self) -> Option<&mut dyn Uart> {
            Some(self)
        }
    }

    fn timing() -> &'static Timing {
        &qemu::RV64_VIRT.timing
    }

    fn run_on(
        output: &str,
        requires: &'static [Requirement],
        steps: &'static [Step],
    ) -> TestOutcome {
        let case = TestCase {
            id: "test",
            requires,
            apps: &[],
            steps,
        };
        let mut console = Console {
            output: ConsoleBuffer::new(),
        };
        console.output.push(output.as_bytes());
        run(&mut console, &case, Path::new("kernel"), &[])
    }

    fn failed(reason: &str) -> TestOutcome {
        TestOutcome::Failed {
            reason: reason.into(),
        }
    }

    #[test]
    fn expects_wait_for_their_text() {
        let requires = &[Requirement::Uart];
        assert_eq!(Expect("a").cost(timing()), timing().expect);
        assert_eq!(run_on("xay", requires, &[Expect("a")]), TestOutcome::Passed);
        assert_eq!(run_on("b", requires, &[Expect("a")]), failed("no \"a\""));
    }

    #[test]
    fn undeclared_resources_fail() {
        assert_eq!(
            run_on("a", &[], &[Expect("a")]),
            failed("test used the console without requiring `Uart`")
        );
    }
}
