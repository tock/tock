// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use crate::ErrorCode;

/// A Tock HIL for talking to an 8042 PS/2 controller.
pub trait PS2Traits {
    /// Block until the controller input buffer is free.
    fn wait_input_ready();

    /// Block until the output buffer has data.
    fn wait_output_ready();

    /// Read a byte from data port.
    fn read_data() -> u8;

    /// Write a command byte to the command port.
    fn write_command(cmd: u8);

    /// Write a data byte to the data port.
    fn write_data(data: u8);

    /// Initialize the controller (self-test, config, enable IRQ, etc).
    fn init(&self);

    /// Called from your IRQ stub.  Should read one byte and
    /// return `Ok(())` or an error code.
    fn handle_interrupt(&self) -> Result<(), ErrorCode>;

    /// Pop one scan‐code out of the driver’s ring buffer.
    fn pop_scan_code(&self) -> Option<u8>;

    /// Push one scan‐code into the ring buffer.
    fn push_code(&self, code: u8) -> Result<(), ErrorCode>;
}
