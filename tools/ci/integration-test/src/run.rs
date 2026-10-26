// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use crate::plan::{Plan, Selection};
use crate::prebuild::{self, BuildArgs};
use crate::test_cases::TestOutcome;
use anyhow::{Result, bail, ensure};
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
    #[arg(long)]
    tock: PathBuf,
    /// Directory of prebuilt artifacts (built into the output directory if
    /// omitted)
    #[arg(long)]
    artifacts_dir: Option<PathBuf>,
    /// libtock-c repository to build the apps in (required to build the
    /// artifacts)
    #[arg(long, conflicts_with = "artifacts_dir")]
    libtock_c: Option<PathBuf>,
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
                let tock = self.tock.clone();
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

        fs::create_dir_all(&logs)?;

        print_banner(&plan);
        let reports = run_tests(&plan, &artifacts, &self.tock, &logs);
        let json = serde_json::to_string_pretty(&reports).unwrap();
        fs::write(&results, json)?;
        println!("{}", summary_table(&reports));

        Ok(
            match reports.iter().all(|r| r.outcome == TestOutcome::Passed) {
                true => ExitCode::SUCCESS,
                false => ExitCode::FAILURE,
            },
        )
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

/// Build a table listing each test's status and the first line of its failure
/// reason.
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

fn run_tests(plan: &Plan, artifacts: &Path, tock: &Path, log_dir: &Path) -> Vec<TestReport> {
    let mut reports = Vec::new();

    for planned in &plan.tests {
        let id = planned.id();
        info!("=== {id} ===");
        let kernel = plan.artifact_path(artifacts, &planned.kernel);
        let apps: Vec<PathBuf> = (planned.apps.iter())
            .map(|label| plan.artifact_path(artifacts, label))
            .collect();
        let apps: Vec<&Path> = apps.iter().map(PathBuf::as_path).collect();

        let mut dut = (planned.board.open)(tock);
        let outcome = crate::test_cases::run(&mut *dut, planned.test, &kernel, &apps);
        if let Some(uart) = dut.uart() {
            write_log(log_dir, &id, uart.transcript());
        }
        if let Err(e) = dut.reset() {
            warn!("failed to reset {}: {e}", planned.board);
        }
        match &outcome {
            TestOutcome::Passed => info!("  PASSED"),
            TestOutcome::Failed { reason } => info!("  FAILED: {reason}"),
            TestOutcome::InfraFailed { reason } => info!("  FAILED (infrastructure): {reason}"),
        }
        reports.push(TestReport { id, outcome });
    }

    reports
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
