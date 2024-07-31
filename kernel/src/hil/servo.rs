// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use crate::ErrorCode;
pub trait Servo<'a> {
    /// Changes the angle of the servo.
    /// Return values:
    ///
    /// - `Ok(())`: The attempt at changing the angle was successful.
    /// - `FAIL`: Cannot change the angle.
    fn servo(&self, angle: usize) -> Result<(), ErrorCode>;
}
