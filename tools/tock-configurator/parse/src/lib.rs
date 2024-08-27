// Copyright OxidOS Automotive 2024.

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

#[cfg(test)]
pub mod test;
