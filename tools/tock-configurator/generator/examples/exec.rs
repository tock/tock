// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use clap::{command, Parser};
use std::error::Error;
use std::path::PathBuf;
use tock_generator::{Nrf52833, TockMain};

/// The arguments for the configuration and the `main.rs` files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct PathArgs {
    #[arg(long)]
    config: PathBuf,

    #[arg(long)]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = PathArgs::parse();
    let tock_main = TockMain::from_json(Nrf52833::default(), args.config)?;
    tock_main.write_to_file(args.output)?;

    Ok(())
}
