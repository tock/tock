// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Interface for SR74HC ShiftRegister
pub trait ShiftRegister {
    /// BITS represents the number of bits supported by the shift register.
    const BITS: usize;

    fn write(&self, value: usize);
}
