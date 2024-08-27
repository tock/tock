// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

pub mod platform;
pub use platform::*;
pub mod component;
pub mod context;
pub use component::*;
pub mod config;
pub use config::Configuration;
pub mod error;

#[macro_use]
mod macros;

pub use error::Error;
pub use parse_macros::component;
pub use parse_macros::peripheral;
pub use proc_macro2;
pub use proc_macro2::TokenStream as Code;
pub use uuid::Uuid;
