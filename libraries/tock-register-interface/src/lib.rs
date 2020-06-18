//! Tock Register Interface
//!
//!

#![feature(const_fn)]
#![cfg_attr(not(feature = "mmio_emu"), no_std)]

pub mod macros;
pub mod registers;

#[cfg_attr(not(feature = "mmio_emu"), path = "mmio.rs")]
#[cfg_attr(feature = "mmio_emu", path = "mmio_emu.rs")]
pub mod mmio;

#[cfg(feature = "mmio_emu")]
mod lazy;
