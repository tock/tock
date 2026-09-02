// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Chip support for the SHAKTI C-Class (RV64) test SoC.
//!
//! Targets the open-source SHAKTI C-Class core (IIT Madras) test SoC
//! as simulated under Verilator. Memory map (from the
//! the bare-metal bring-up path):
//!
//! - RAM   `0x8000_0000` (2 GiB)
//! - CLINT `0x0200_0000` (SiFive-*like*, but needs 64-bit `mtime` accesses; see [`clint`])
//! - UART  `0x0001_1300` (custom SHAKTI UART; see [`uart`])
//!
//! The test SoC ties the PLIC `meip`/`seip` inputs to zero, so there is no
//! external interrupt controller: peripherals are polled and only the CLINT
//! (machine timer / software) interrupts are used.

#![no_std]

pub mod chip;
pub mod clint;
pub mod uart;
