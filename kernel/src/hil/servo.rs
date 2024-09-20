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
    /// - `INVAL`: The value exceeds u16, indicating it's incorrect
    /// since servomotors can only have a maximum of 360 degrees.
    /// - `NODEVICE`: The index exceeds the number of servomotors provided.
    ///  # Arguments
    /// - `angle` - the variable that receives the angle
    /// (in degrees from 0 to 180) from the servo driver.
    fn set_angle(&self, angle: u16) -> Result<(), ErrorCode>;

    /// Returns the angle of the servo.
    /// Return values:
    ///
    /// - `angle`: The value, in angles from 0 to 360, of the servo.
    /// - `NOSUPPORT`:  The servo cannot return its angle.
    /// - `NODEVICE`: The index exceeds the number of servomotors provided.
    fn get_angle(&self) -> Result<usize, ErrorCode>;
}
