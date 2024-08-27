// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

//! Utils used for the configurator TUI.

mod capsule;
mod menu;
mod state;
mod utils;

// Reimports
pub use menu::init_configurator as init;
pub(crate) use utils::*;
