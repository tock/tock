// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Mock implementation of the ISL29035 digital ambient light sensor.
//!
//! Following the same approach as [`crate::mock::sensors::sht4x`], this capsule
//! pretends to be a real ISL29035 chip on an I2C bus so the
//! [`crate::isl29035`] driver (and the ambient-light syscall driver on top of
//! it) can be exercised with no physical bus and no physical sensor.
//!
//! It implements [`kernel::hil::i2c::I2CDevice`] and, via
//! [`MockI2CDevice`](crate::mock::i2c_bus::MockI2CDevice), attaches to a
//! [`MockI2CBus`](crate::mock::i2c_bus::MockI2CBus). A [`DeferredCall`] stands
//! in for the controller's transaction-complete interrupt.
//!
//! The mock watches the register the driver addresses: configuration and
//! power-down writes are acknowledged, and a read of the data register returns
//! a raw ADC count chosen so the driver's `lux = (count * 4000) >> 8` decodes
//! back to the configured [`set_lux`](MockIsl29035::set_lux) value. The driver
//! runs the ADC at 8-bit resolution, so the reported level is quantised to
//! steps of `4000 / 256 ~= 15.6` lux.

use core::cell::Cell;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::i2c;
use kernel::utilities::cells::{OptionalCell, TakeCell};

/// I2C bus address of the (real) ISL29035. The `isl29035` driver hard-codes
/// this address.
pub const BASE_ADDR: u8 = 0x44;

/// Register the `isl29035` driver reads the ADC output from (data LSB; the
/// MSB at `0x03` follows via address auto-increment).
const REG_DATA_LSB: u8 = 0x02;

/// Full-scale lux range the `isl29035` driver configures (`CMD2` selects
/// range 4000).
const RANGE_LUX: usize = 4000;
/// ADC full-scale count at the 8-bit resolution the driver configures.
const ADC_FULL_SCALE: usize = 0xFF;

/// Default ambient light level the mock reports, in lux. (500 lx maps exactly
/// onto an 8-bit ADC count.)
const DEFAULT_LUX: usize = 500;

pub struct MockIsl29035<'a> {
    client: OptionalCell<&'a dyn i2c::I2CClient>,
    /// Buffer for the in-flight transaction, returned to the client when the
    /// deferred call fires.
    buffer: TakeCell<'static, [u8]>,
    /// Deferred call standing in for the "transaction complete" interrupt.
    deferred_call: DeferredCall,
    /// Result to report for the in-flight transaction.
    status: Cell<Result<(), i2c::Error>>,
    /// Reported ambient light level, in lux.
    lux: Cell<usize>,
}

impl Default for MockIsl29035<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MockIsl29035<'a> {
    pub fn new() -> Self {
        Self {
            client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            deferred_call: DeferredCall::new(),
            status: Cell::new(Ok(())),
            lux: Cell::new(DEFAULT_LUX),
        }
    }

    /// Register the I2C completion-callback client.
    pub fn set_client(&'a self, client: &'a dyn i2c::I2CClient) {
        self.client.set(client);
    }

    /// Set the ambient light level the mock reports, in lux.
    pub fn set_lux(&self, lux: usize) {
        self.lux.set(lux);
    }

    /// Schedule the deferred call standing in for the transaction-complete
    /// interrupt.
    fn finish(&self, data: &'static mut [u8], status: Result<(), i2c::Error>) {
        self.buffer.replace(data);
        self.status.set(status);
        self.deferred_call.set();
    }

    /// Raw ADC count that decodes to the configured lux, inverting the
    /// driver's `lux = (count * RANGE_LUX) >> 8`.
    fn adc_count(&self) -> u8 {
        let count = ((self.lux.get() << 8) + RANGE_LUX / 2) / RANGE_LUX;
        count.min(ADC_FULL_SCALE) as u8
    }

    /// Fill `buf` with the response a real ISL29035 would clock out for a read
    /// starting at register `reg`.
    fn fill_response(&self, reg: u8, buf: &mut [u8]) {
        if reg == REG_DATA_LSB && buf.len() >= 2 {
            buf[0] = self.adc_count(); // data LSB (the only byte the driver uses)
            buf[1] = 0; // data MSB (ignored at 8-bit resolution)
        }
    }

    fn perform_transaction(
        &self,
        data: &'static mut [u8],
        reg: Option<u8>,
        read_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        if self.buffer.is_some() {
            return Err((i2c::Error::Busy, data));
        }

        if let Some(reg) = reg {
            let end = read_len.min(data.len());
            self.fill_response(reg, &mut data[..end]);
        }
        self.finish(data, Ok(()));
        Ok(())
    }
}

impl i2c::I2CDevice for MockIsl29035<'_> {
    fn enable(&self) {}

    fn disable(&self) {}

    fn write(
        &self,
        data: &'static mut [u8],
        _write_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        // Configuration / power-down register writes: just acknowledge.
        self.perform_transaction(data, None, 0)
    }

    fn read(
        &self,
        buffer: &'static mut [u8],
        _read_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        // The `isl29035` driver always uses `write_read`; a bare read has no
        // register context, so just acknowledge.
        self.perform_transaction(buffer, None, 0)
    }

    fn write_read(
        &self,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        let reg = (write_len >= 1).then(|| data[0]);
        self.perform_transaction(data, reg, read_len)
    }
}

impl<'a> crate::mock::i2c_bus::MockI2CDevice<'a> for MockIsl29035<'a> {
    fn set_i2c_client(&'a self, client: &'a dyn i2c::I2CClient) {
        self.set_client(client);
    }
}

impl DeferredCallClient for MockIsl29035<'_> {
    fn handle_deferred_call(&self) {
        let status = self.status.get();
        if let Some(buffer) = self.buffer.take() {
            self.client
                .map(move |client| client.command_complete(buffer, status));
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
