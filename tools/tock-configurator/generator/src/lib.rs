// Copyright OxidOS Automotive 2024.

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
