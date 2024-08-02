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
    ///
    /// "&self" is used so the function can modify the servomotor.
    /// "angle" is the parameter that receives the angle
    /// (in degrees from 0 to 180) from the servo driver.
    fn servo(&self, angle: usize) -> Result<(), ErrorCode>;
}
