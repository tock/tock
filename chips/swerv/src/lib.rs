// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Drivers and support modules for SweRV SoCs

#![no_std]
#![crate_name = "swerv"]
#![crate_type = "rlib"]

pub mod eh1_pic;
pub mod eh1_timer;
