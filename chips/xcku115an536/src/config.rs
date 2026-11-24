// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Specific device definition for the MPS3-AN536 instance pf the Cortex R52.
//! 
//! All data come from the document "Example Cortex™-R52x2 Subsystem for MPS3
//! Application Note AN536 / Issue B / DAI 0536"
//! This file is supposed to be included by this crate but also directly 
//! by the arch crate cortexr52.

use kernel::hil::time::Frequency;

//
// CLOCKS
//

// Main clock
pub const FREQ_ACLK: u32 = 50_000_000;
// RTC and DualTimer clock
pub const FREQ_TCLK: u32 = 24_000_000;

/// Frequency struct for main clock
pub struct FreqAclk;
impl Frequency for FreqAclk {
    fn frequency() -> u32 {
        FREQ_ACLK
    }
}

/// Frequency struct for timer clock
pub struct FreqTclk;
impl Frequency for FreqTclk {
    fn frequency() -> u32 {
        FREQ_TCLK
    }
}

//
// PERIPHERALS ADDRESSES
//

/// GICv3 distributor (GICD) registers
pub const BASE_GICD: u32 = 0xf000_0000;

// UARTs registers
// Instances 0 and 1 are per core only and use the same Low Latency Peripheral Port address
pub const BASE_UART01: u32 = 0xE7C0_0000;
// Instances 2 is the only one tested.
// Qemu command line should include: "-serial none -serial none -serial mon:stdio"
pub const BASE_UART2: u32 = 0xE020_5000;
pub const BASE_UART3: u32 = 0xE020_6000;
pub const BASE_UART4: u32 = 0xE020_7000;
pub const BASE_UART5: u32 = 0xE020_8000;

pub const BASE_DUALTIMER1: u32 = 0xE010_1000;
pub const BASE_DUALTIMER2: u32 = 0xE010_1020;
