// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

mod boards;
mod plan;
mod prebuild;
mod run;
mod test_cases;

use clap::Parser;
use std::process::ExitCode;

/// Tock integration test harness
#[derive(Parser)]
struct Cli {
    /// Increase log verbosity (`-v` for debug and `-vv` for trace)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Step 1: generate a plan of tests and compute the artifacts they require
    Plan(plan::Args),
    /// Step 2: build the artifacts of a plan (kernels, applications, ...)
    Prebuild(prebuild::Args),
    /// Step 3: run the tests of a plan on a board attached to this host
    Run(run::Args),
}

fn main() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse();

    let level = match cli.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_level(level)
        .parse_default_env()
        .format_timestamp(None)
        .format_target(false)
        .format_indent(Some(8))
        .init();

    match cli.command {
        Subcommand::Plan(args) => args.run().map(|()| ExitCode::SUCCESS),
        Subcommand::Prebuild(args) => args.run().map(|()| ExitCode::SUCCESS),
        Subcommand::Run(args) => args.run(),
    }
}
