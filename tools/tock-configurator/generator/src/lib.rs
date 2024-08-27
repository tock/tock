// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

//! A code generator for **TockOS** platforms' `main.rs` files.
//!
//! The generator takes as input a *TockOS configuration file* in JSON format,
//! and generates a `main.rs` board file.  
//!
//! ## Example
//! ```rust,ignore
//! use tock_generator::{TockMain, Nrf52833};
//!
//! // This errors should be handled.
//! let tock_main = TockMain::from_json(Nrf52833::default(), ".config.json")?;
//! tock_main.write_to_file("main.rs")?;
//! ```

mod tock_main;
mod util;

pub use nrf52833::Chip as Nrf52833;
pub use tock_main::TockMain;
