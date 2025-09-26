// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Tock PCI support library for x86 devices
//!
//! This crate provides types and helpers for working with PCI devices within Tock OS kernel
//! drivers. It specifically targets the PCI Local Bus Specification, Revision 3.0, as published by
//! the PCI-SIG. All references to the PCI specification throughout this crate's documentation and
//! comments refer to this document.
//!
//! Limitations:
//!
//! * No support for PCI Express of any revision.
//! * x86 only, using I/O ports to access configuration registers.

#![no_std]

mod bdf;
pub use self::bdf::Bdf;

pub mod cap;

pub mod cfg;

mod device;
pub use self::device::{Command, CommandVal, Device, Status, StatusVal};

mod iter;
pub use self::iter::iter;
