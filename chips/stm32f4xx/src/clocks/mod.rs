// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

pub mod clocks;
pub mod hsi;
pub mod pll;

pub use crate::clocks::clocks::tests;
pub use crate::clocks::clocks::Clocks;
