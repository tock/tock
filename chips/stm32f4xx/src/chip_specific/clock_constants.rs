// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL.
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

//! Clock-related constants for a particular chip

/// PLL-related constants for specific for a specific chip
pub trait PllConstants {
    /// PLL minimum frequency in MHz
    const MIN_FREQ_MHZ: usize;
    /// PLL maximum frequency in MHz
    // All boards support PLL frequencies up to 216MHz
    const MAX_FREQ_MHZ: usize = 216;
}

/// Generic clock constants for a specific chip
pub trait SystemClockConstants {
    /// Maximum allowed APB1 frequency in MHz
    const APB1_FREQUENCY_LIMIT_MHZ: usize;
    /// Maximum allowed APB2 frequency in MHz
    // APB2 frequency limit is twice the APB1 frequency limit
    const APB2_FREQUENCY_LIMIT_MHZ: usize = Self::APB1_FREQUENCY_LIMIT_MHZ << 1;
    /// Maximum allowed system clock frequency in MHz
    const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize;
}

/// Clock constants for a specific chip
pub trait ClockConstants: SystemClockConstants + PllConstants {}

impl<T: SystemClockConstants + PllConstants> ClockConstants for T {}
