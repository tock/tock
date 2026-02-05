// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

//! Clock-related constants for a particular chip

/// PLL-related constants for specific for a specific chip
pub trait PllConstants {
    /// PLL minimum frequency in MHz
    const MIN_FREQ_MHZ: usize = 8;
    /// PLL maximum frequency in MHz
    // All boards support PLL frequencies up to 80MHz
    const MAX_FREQ_MHZ: usize = 80;
}

/// Generic clock constants for a specific chip
pub trait SystemClockConstants {
    /// Maximum allowed APB1 frequency in MHz
    const APB1_FREQUENCY_LIMIT_MHZ: usize = 80;
    /// Maximum allowed APB2 frequency in MHz
    const APB2_FREQUENCY_LIMIT_MHZ: usize = 80;
    /// Maximum allowed system clock frequency in MHz
    const SYS_CLOCK_FREQUENCY_LIMIT_MHZ: usize = 80;
}

/// Clock constants for a specific chip
pub trait ClockConstants: SystemClockConstants + PllConstants {}

impl<T: SystemClockConstants + PllConstants> ClockConstants for T {}
