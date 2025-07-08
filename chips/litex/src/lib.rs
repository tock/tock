// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Drivers and support modules for LiteX SoCs

#![no_std]

// Exported as the LiteX Register Abstraction may be used by other
// modules
pub mod litex_registers;

pub mod event_manager;
pub mod gpio;
pub mod led_controller;
pub mod liteeth;
pub mod timer;
pub mod uart;
