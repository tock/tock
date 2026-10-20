// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Chip support for the ARM MPS3-AN536 (Cortex-R52) platform.

#![no_std]

pub mod an536_inst;
pub mod chip;
pub mod dualtimer;
pub mod uart;
