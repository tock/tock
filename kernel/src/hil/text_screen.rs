// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Interface for text screen and displays.

use crate::ErrorCode;

pub trait TextScreen<'a> {
    fn set_client(&self, client: Option<&'a dyn TextScreenClient>);

    /// Returns a tuple (width, height) with the resolution of the
    /// screen that is being used. This function is synchronous as the
    /// resolution is known by the driver at any moment.
    ///
    /// The resolution is constant.
    fn get_size(&self) -> (usize, usize);

    /// Sends a write command to the driver, and the buffer to write from
    /// and the len are sent as arguments. When the `write` operation is
    /// finished, the driver will call the `write_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The write command is valid and will be sent to the driver.
    /// - `BUSY`: The driver is busy with another command.
    fn print(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Sends to the driver a command to set the cursor at a given position
    /// (x_position, y_position). When finished, the driver will call the
    /// `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn set_cursor(&self, x_position: usize, y_position: usize) -> Result<(), ErrorCode>;

    /// Sends to the driver a command to hide the cursor. When finished,
    /// the driver will call the `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn hide_cursor(&self) -> Result<(), ErrorCode>;

    /// Sends to the driver a command to show the cursor. When finished,
    /// the driver will call the `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn show_cursor(&self) -> Result<(), ErrorCode>;

    /// Sends to the driver a command to turn on the blinking cursor. When finished,
    /// the driver will call the `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn blink_cursor_on(&self) -> Result<(), ErrorCode>;

    /// Sends to the driver a command to turn off the blinking cursor. When finished,
    /// the driver will call the `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn blink_cursor_off(&self) -> Result<(), ErrorCode>;

    /// Sends to the driver a command to turn on the display of the screen.
    /// When finished, the driver will call the `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn display_on(&self) -> Result<(), ErrorCode>;

    /// Sends to the driver a command to turn off the display of the screen.
    /// When finished, the driver will call the `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn display_off(&self) -> Result<(), ErrorCode>;

    /// Sends to the driver a command to clear the display of the screen.
    /// When finished, the driver will call the `command_complete()` callback.
    ///
    /// Return values:
    /// - `Ok(())`: The command is valid and will be sent to the driver.
    /// - `BUSY`: Another command is in progress.
    fn clear(&self) -> Result<(), ErrorCode>;
}

pub trait TextScreenClient {
    /// The driver calls this function when any command (but a write one)
    /// finishes executing.
    fn command_complete(&self, r: Result<(), ErrorCode>);

    /// The driver calls this function when a write command finishes executing.
    fn write_complete(&self, buffer: &'static mut [u8], len: usize, r: Result<(), ErrorCode>);
}
