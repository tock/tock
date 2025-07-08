// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! LiteX SoCs based around a VexRiscv CPU

#![no_std]

pub use litex::{event_manager, gpio, led_controller, liteeth, litex_registers, timer, uart};

pub mod chip;
pub mod interrupt_controller;
