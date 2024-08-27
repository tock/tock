// Copyright OxidOS Automotive 2024.

//! Utils used for the configurator TUI.

mod capsule;
mod menu;
mod state;
mod utils;

// Reimports
pub use menu::init_configurator as init;
pub(crate) use utils::*;
