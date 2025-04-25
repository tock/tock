// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Support for traditional x86 PC hardware.
//!
//! ## Safety
//!
//! This crate inherits all of the same safety hazards as outlined by [`x86`].
//!
//! Additionally, this crate assumes the presence of certain traditional/legacy PC peripherals such
//! as serial ports and interrupt controllers.

#![deny(unsafe_op_in_unsafe_fn)]
#![no_std]

mod chip;
pub use chip::{Pc, PcComponent};

mod interrupts;

mod pic;

pub mod pit;

pub mod serial;
