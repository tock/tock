// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for digital to analog converters.

use crate::ErrorCode;

/// Simple interface for using the DAC.
pub trait DacChannel {
    /// Set the DAC output value.
    fn set_value(&self, value: usize) -> Result<(), ErrorCode>;
}
