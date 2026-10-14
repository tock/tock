// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use crate::boards::{AttachedDut, Dut, Open};
use crate::plan::{Plan, PlannedTest, Selection};
use crate::prebuild::{self, BuildArgs};
use crate::test_cases::{Requirement, TestOutcome};
use anyhow::{Context, Result, bail, ensure};
use fs_err as fs;
use log::{info, warn};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::SystemTime;

#[derive(clap::Args)]
pub struct Args {
    /// Plan to run (computed from `--test` if omitted)
    #[arg(long)]
    plan: Option<PathBuf>,
    /// Tock repository to build the kernels and run the virtual boards in
    /// (required to build the artifacts or run a virtual board)
    #[arg(long)]
    tock: Option<PathBuf>,
    /// Directory of prebuilt artifacts (built into the output directory if
    /// omitted)
    #[arg(long)]
    artifacts_dir: Option<PathBuf>,
    /// libtock-c repository to build the apps in (required to build the
    /// artifacts)
    #[arg(long, conflicts_with = "artifacts_dir")]
    libtock_c: Option<PathBuf>,
    /// DUT attached to this host, as
    /// `<board>[,probe=<serial>][,console=<device>]` (may be repeated)
    #[arg(long, value_name = "DUT")]
    dut: Vec<AttachedDut>,
    /// Run only part of the plan
    #[command(flatten)]
    selection: Selection,
    /// Directory to write the results of the tests, their console transcripts
    /// and the built artifacts to
    #[arg(long)]
    out_dir: PathBuf,
}

/// File in the output directory which holds the results of the tests.
pub const RESULTS_FILE: &str = "results.json";
/// Directory in the output directory which holds each test's console
/// transcript.
pub const LOGS_DIR: &str = "logs";
/// Directory in the output directory which `run` builds the artifacts into,
/// if it is given no prebuilt artifacts.
const ARTIFACTS_DIR: &str = "artifacts";

/// Return the paths of the results file and the logs directory in `out_dir`.
///
/// This fails if either exists already, as the results and console transcripts
/// of a run would otherwise mix with those of an earlier run.
pub fn output_paths(out_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    let results = out_dir.join(RESULTS_FILE);
    let logs = out_dir.join(LOGS_DIR);
    for path in [&results, &logs] {
        ensure!(!path.exists(), "{} already exists", path.display());
    }
    Ok((results, logs))
}

impl Args {
    /// Run the selected tests of a plan and write their reports to the output
    /// directory.
    ///
    /// This returns a failure exit code if any test failed.
    pub fn run(self) -> Result<ExitCode> {
        let plan = match &self.plan {
            Some(path) => Plan::load(path)?.select(&self.selection)?,
            None if self.selection.tests.is_empty() => bail!("either --plan or --test is required"),
            None => Plan::compute(&self.selection)?,
        };
        let (results, logs) = output_paths(&self.out_dir)?;
        let artifacts = match (self.artifacts_dir, self.libtock_c) {
            (Some(dir), _) => dir,
            (None, Some(libtock_c)) => {
                let tock =
                    (self.tock.clone()).context("--tock is required to build the artifacts")?;
                let build = BuildArgs {
                    tock,
                    libtock_c,
                    artifacts_dir: self.out_dir.join(ARTIFACTS_DIR),
                };
                prebuild::build(&plan, &build)?;
                build.artifacts_dir
            }
            (None, None) => bail!("either --artifacts-dir or --libtock-c is required"),
        };

        // Virtual boards run from their board directory in the Tock repository:
        let tock = self.tock.as_deref();
        let virtual_test = (plan.tests.iter()).find(|t| matches!(t.board.open, Open::Virtual(_)));
        if let (None, Some(planned)) = (tock, virtual_test) {
            bail!(
                "--tock is required to run {}, which is virtual",
                planned.board
            );
        }

        fs::create_dir_all(&logs)?;

        print_banner(&plan);
        let reports = run_tests(&plan, &artifacts, tock, &self.dut, &logs);
        let json = serde_json::to_string_pretty(&reports).unwrap();
        fs::write(&results, json)?;
        println!("{}", summary_table(&reports));

        let failed = reports.iter().any(|r| {
            matches!(
                r.outcome,
                TestOutcome::Failed { .. } | TestOutcome::InfraFailed { .. }
            )
        });
        Ok(match failed {
            true => ExitCode::FAILURE,
            false => ExitCode::SUCCESS,
        })
    }
}

#[derive(Serialize)]
struct TestReport {
    id: String,
    outcome: TestOutcome,
}

/// Width of the summary table.
const SUMMARY_WIDTH: u16 = 80;
/// Maximum number of characters of a failure reason in the summary table.
/// Longer reasons are truncated with an ellipsis.
const SUMMARY_REASON_CHARS: usize = 200;

/// Build a table listing each test's status and the first line of the reason
/// for its outcome, if any.
fn summary_table(reports: &[TestReport]) -> comfy_table::Table {
    let mut table = comfy_table::Table::new();
    table.load_style(comfy_table::presets::ASCII_BORDERS_ONLY_CONDENSED);
    table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
    table.set_width(SUMMARY_WIDTH);
    table.set_header(["test", "status", "reason"]);
    for report in reports {
        let (status, reason) = match &report.outcome {
            TestOutcome::Passed => ("passed", ""),
            TestOutcome::Failed { reason } => ("FAILED", reason.as_str()),
            TestOutcome::InfraFailed { reason } => ("INFRA", reason.as_str()),
            TestOutcome::Skipped { reason } => ("skipped", reason.as_str()),
        };
        let mut reason = reason.lines().next().unwrap_or("").to_string();
        if let Some((cut, _)) = reason.char_indices().nth(SUMMARY_REASON_CHARS) {
            reason.truncate(cut);
            reason.push('…');
        }
        table.add_row([report.id.as_str(), status, &reason]);
    }
    if let Some(column) = table.column_mut(0) {
        let half = comfy_table::Width::Percentage(50);
        column.set_constraint(comfy_table::ColumnConstraint::UpperBoundary(half));
    }
    table
}

fn run_tests(
    plan: &Plan,
    artifacts: &Path,
    tock: Option<&Path>,
    duts: &[AttachedDut],
    log_dir: &Path,
) -> Vec<TestReport> {
    let mut reports = Vec::new();

    for planned in &plan.tests {
        let id = planned.id();
        info!("=== {id} ===");
        let kernel = plan.artifact_path(artifacts, &planned.kernel);
        let apps: Vec<PathBuf> = (planned.apps.iter())
            .map(|label| plan.artifact_path(artifacts, label))
            .collect();
        let apps: Vec<&Path> = apps.iter().map(PathBuf::as_path).collect();

        let outcome = match open_dut(planned, tock, duts) {
            Ok(mut dut) => {
                let outcome = crate::test_cases::run(&mut *dut, planned.test, &kernel, &apps);
                if let Some(uart) = dut.uart() {
                    write_log(log_dir, &id, uart.transcript());
                }
                if let Err(e) = dut.reset() {
                    warn!("failed to reset {}: {e:#}", planned.board);
                }
                outcome
            }
            Err(e) => TestOutcome::Skipped {
                reason: format!("{e:#}"),
            },
        };
        match &outcome {
            TestOutcome::Passed => info!("  PASSED"),
            TestOutcome::Failed { reason } => info!("  FAILED: {reason}"),
            TestOutcome::InfraFailed { reason } => info!("  FAILED (infrastructure): {reason}"),
            TestOutcome::Skipped { reason } => info!("  SKIPPED: {reason}"),
        }
        reports.push(TestReport { id, outcome });
    }

    reports
}

/// Open a board to run `planned` on.
///
/// A physical board is bound to the first of the `duts` which satisfies all
/// requirements of the test, such that a test always runs on the same DUT. If
/// there is no such DUT, this returns the reason, which lists the requirements
/// that the first DUT lacking the fewest of them lacks, and the test is skipped.
///
/// A virtual board is opened from the Tock repository `tock`, which must be
/// given.
fn open_dut(
    planned: &PlannedTest,
    tock: Option<&Path>,
    duts: &[AttachedDut],
) -> Result<Box<dyn Dut>> {
    let board = planned.board;
    let open = match board.open {
        Open::Virtual(open) => {
            let tock = tock.with_context(|| format!("--tock is required to run {board}"))?;
            return Ok(open(tock));
        }
        Open::Physical(open) => open,
    };
    let requires = planned.test.requires;
    let closest = (duts.iter())
        .filter(|dut| dut.board == board)
        .map(|dut| (dut, dut.missing(requires)))
        .min_by_key(|(_, missing)| missing.len());
    let Some((dut, missing)) = closest else {
        bail!("no {board} DUT given with --dut");
    };
    if !missing.is_empty() {
        let missing: Vec<String> = missing.iter().map(Requirement::describe).collect();
        bail!(
            "no {board} DUT given provides all requirements, missing: {}",
            missing.join(", ")
        );
    }
    Ok(open(dut.config.clone()))
}

/// Write a test's console transcript to `<dir>/<id>.log`.
///
/// Failures are only logged as warnings, as the test's outcome does not depend
/// on the transcript.
fn write_log(dir: &Path, id: &str, transcript: &[u8]) {
    let path = dir.join(format!("{id}.log"));
    match fs::write(&path, transcript) {
        Ok(()) => info!("  console log: {}", path.display()),
        Err(e) => warn!("{e}"),
    }
}

/// Print a banner announcing the run and summarizing `plan`.
fn print_banner(plan: &Plan) {
    let rule = "=".repeat(72);
    info!("{rule}");
    info!(
        "tock-integration-test-harness {} run",
        env!("CARGO_PKG_VERSION")
    );
    info!(
        "  time:     {}",
        humantime::format_rfc3339_seconds(SystemTime::now())
    );
    for line in plan.summary().lines() {
        info!("  {line}");
    }
    info!("{rule}");
}
