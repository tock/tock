// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! ADC driver, nRF5x-family, unsafe code

use core::ptr::{addr_of, read_volatile};

/// Where the SAADC stores a single sample.
static mut SAMPLE: u16 = 0;

/// Owner of the buffer the SAADC writes single samples to.
pub struct AdcSample(());

impl AdcSample {
    /// # Safety
    ///
    /// At most one may exist, as there is only one buffer.
    pub const unsafe fn new() -> Self {
        Self(())
    }

    /// The address to give the SAADC as its result pointer.
    pub fn address(&self) -> u32 {
        addr_of!(SAMPLE) as u32
    }

    /// The sample the SAADC last wrote.
    pub fn read(&self) -> i16 {
        // # Safety
        //
        // The SAADC writes this aligned `u16` by DMA, so a volatile read sees
        // a whole sample, and no reference to it ever exists.
        unsafe { read_volatile(addr_of!(SAMPLE)) as i16 }
    }
}
