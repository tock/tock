// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Mock implementation of the Sensirion SHT4x temperature and humidity sensor.
//!
//! This capsule pretends to be a real SHT4x chip sitting on an I2C bus, so the
//! [`crate::sht4x`] driver (and everything layered on top of it) can be
//! exercised with no physical bus and no physical sensor.
//!
//! It implements the [`kernel::hil::i2c::I2CDevice`] HIL.
//!
//! Because there is no real bus, there are also no real interrupts. Instead
//! the mock uses a [`DeferredCall`] to asynchronously invoke the client's
//! `command_complete`, mimicking the interrupt the I2C controller would raise
//! when a transaction finishes.
//!
//! The mock behaves like the chip itself: it inspects the raw bytes the driver
//! "writes" onto the bus, decodes the SHT4x command in them, and, when the
//! driver "reads" back, synthesizes the raw ADC words (including the per-word
//! CRC) that a real SHT4x would return. The reported temperature and humidity
//! are configurable via [`MockSht4x::set_temperature`] and
//! [`MockSht4x::set_humidity`].
//!
//! Usage
//! -----
//!
//! Behind the I2C virtualizer (the mock stands in for the controller):
//!
//! ```rust,ignore
//! let mock_i2c = static_init!(
//!     capsules_extra::mock::sensors::sht4x::MockSht4x<'static>,
//!     capsules_extra::mock::sensors::sht4x::MockSht4x::new()
//! );
//! mock_i2c.register();
//!
//! let mux_i2c = components::i2c::I2CMuxComponent::new(mock_i2c, None)
//!     .finalize(components::i2c_mux_component_static!(
//!         capsules_extra::mock::sensors::sht4x::MockSht4x<'static>
//!     ));
//!
//! let sht4x = components::sht4x::SHT4xComponent::new(
//!     mux_i2c,
//!     capsules_extra::mock::sensors::sht4x::BASE_ADDR,
//!     mux_alarm,
//! )
//! .finalize(components::sht4x_component_static!(
//!     ChipAlarm,
//!     capsules_extra::mock::sensors::sht4x::MockSht4x<'static>
//! ));
//! ```

use core::cell::Cell;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::i2c::{self, I2CHwMasterClient};
use kernel::utilities::cells::{OptionalCell, TakeCell};

/// I2C bus address of the (real) SHT4x. Matches [`crate::sht4x::BASE_ADDR`].
pub const BASE_ADDR: u8 = 0x44;

// SHT4x command bytes. These are the first (and, for the commands the driver
// issues, only) byte the driver writes onto the bus.
const CMD_MEAS_HIGH_REP: u8 = 0xFD;
const CMD_MEAS_MED_REP: u8 = 0xF6;
const CMD_MEAS_LOW_REP: u8 = 0xE0;
const CMD_READ_SERIAL: u8 = 0x89;
const CMD_SOFT_RESET: u8 = 0x94;

/// Default temperature the mock reports, in hundredths of a degree Celsius
/// (22.00 C).
const DEFAULT_TEMPERATURE_CENTI_C: i32 = 2200;
/// Default relative humidity the mock reports, in hundredths of a percent
/// (50.00 %RH).
const DEFAULT_HUMIDITY_CENTI_PCT: u32 = 5000;
/// Fake 32-bit serial number handed back for the "read serial number" command.
const DEFAULT_SERIAL: u32 = 0x0BAD_5417;

/// CRC-8 as used by the SHT4x (polynomial 0x31, initial value 0xFF). This is
/// the same algorithm the real driver uses to check the data it reads back.
fn crc8(data: &[u8]) -> u8 {
    let polynomial = 0x31;
    let mut crc: u8 = 0xff;

    for byte in data {
        crc ^= *byte;
        for _ in 0..8 {
            if (crc & 0x80) != 0 {
                crc = (crc << 1) ^ polynomial;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Which SHT4x transaction the mock is part-way through. Set when the driver
/// writes a command, consumed when it reads the response.
#[derive(Clone, Copy, PartialEq)]
enum Transaction {
    /// No command has been written, or the last one needs no response.
    None,
    /// A measurement command was written; the next read returns 6 bytes of
    /// temperature + humidity data.
    Measure,
    /// The read-serial-number command was written; the next read returns the
    /// 6-byte serial number response.
    Serial,
}

pub struct MockSht4x<'a> {
    /// Client for when the mock is wired directly as an [`i2c::I2CDevice`].
    client: OptionalCell<&'a dyn i2c::I2CClient>,
    /// Client for when the mock stands in for the I2C controller and has the
    /// virtualizer ([`MuxI2C`](capsules_core::virtualizers::virtual_i2c::MuxI2C))
    /// stacked on top of it. Only one of `client` / `master_client` is ever in
    /// use for a given board.
    master_client: OptionalCell<&'a dyn I2CHwMasterClient>,
    /// Buffer handed to us for the in-flight transaction, returned to the
    /// client when the deferred call fires.
    buffer: TakeCell<'static, [u8]>,
    /// Deferred call used to stand in for the "transaction complete" interrupt.
    deferred_call: DeferredCall,
    /// Result to report for the in-flight transaction.
    status: Cell<Result<(), i2c::Error>>,
    /// The command the driver last wrote, awaiting its read.
    pending: Cell<Transaction>,
    /// Reported temperature, hundredths of a degree Celsius.
    temperature: Cell<i32>,
    /// Reported relative humidity, hundredths of a percent.
    humidity: Cell<u32>,
    /// Serial number returned for `CMD_READ_SERIAL`.
    serial: Cell<u32>,
}

impl Default for MockSht4x<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MockSht4x<'a> {
    pub fn new() -> Self {
        Self {
            client: OptionalCell::empty(),
            master_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            deferred_call: DeferredCall::new(),
            status: Cell::new(Ok(())),
            pending: Cell::new(Transaction::None),
            temperature: Cell::new(DEFAULT_TEMPERATURE_CENTI_C),
            humidity: Cell::new(DEFAULT_HUMIDITY_CENTI_PCT),
            serial: Cell::new(DEFAULT_SERIAL),
        }
    }

    /// Register the client for the [`i2c::I2CDevice`] wiring. Mirrors the
    /// inherent `set_client` on the real virtualized
    /// [`I2CDevice`](capsules_core::virtualizers::virtual_i2c::I2CDevice).
    pub fn set_client(&'a self, client: &'a dyn i2c::I2CClient) {
        self.client.set(client);
    }

    /// Set the temperature the mock will report, in hundredths of a degree
    /// Celsius (e.g. `2537` for 25.37 C).
    pub fn set_temperature(&self, centi_celsius: i32) {
        self.temperature.set(centi_celsius);
    }

    /// Set the relative humidity the mock will report, in hundredths of a
    /// percent (e.g. `4200` for 42.00 %RH).
    pub fn set_humidity(&self, centi_percent: u32) {
        self.humidity.set(centi_percent);
    }

    /// Set the serial number returned for the read-serial-number command.
    pub fn set_serial(&self, serial: u32) {
        self.serial.set(serial);
    }

    /// Schedule the deferred call that stands in for the transaction-complete
    /// interrupt. `data` is the buffer to hand back to the client; `status`
    /// is the result to report.
    fn finish(&self, data: &'static mut [u8], status: Result<(), i2c::Error>) {
        self.buffer.replace(data);
        self.status.set(status);
        self.deferred_call.set();
    }

    /// Act on a command byte the driver wrote onto the "bus".
    fn handle_command(&self, cmd: u8) {
        match cmd {
            CMD_MEAS_HIGH_REP | CMD_MEAS_MED_REP | CMD_MEAS_LOW_REP => {
                self.pending.set(Transaction::Measure);
            }
            CMD_READ_SERIAL => {
                self.pending.set(Transaction::Serial);
            }
            CMD_SOFT_RESET => {
                self.pending.set(Transaction::None);
            }
            _ => {
                // Unknown command: a real chip would just NAK or ignore it.
                self.pending.set(Transaction::None);
            }
        }
    }

    /// Fill `buffer` with the bytes a real SHT4x would clock out in response
    /// to the pending command. Returns the number of bytes produced.
    fn fill_response(&self, buffer: &mut [u8]) -> usize {
        match self.pending.get() {
            Transaction::Measure => {
                if buffer.len() < 6 {
                    return 0;
                }

                // Invert the fixed-point conversions the driver applies so
                // that it decodes back to our configured values:
                //   T_centi  = ((4375 * raw) >> 14) - 4500
                //   RH_centi = (625 * raw) >> 12
                let t_raw = (((self.temperature.get() as i64) + 4500) << 14) / 4375;
                let t_raw = t_raw.clamp(0, 0xFFFF) as u16;

                let rh_raw = ((self.humidity.get() as u64) << 12) / 625;
                let rh_raw = rh_raw.min(0xFFFF) as u16;

                buffer[0] = (t_raw >> 8) as u8;
                buffer[1] = t_raw as u8;
                buffer[2] = crc8(&buffer[0..2]);
                buffer[3] = (rh_raw >> 8) as u8;
                buffer[4] = rh_raw as u8;
                buffer[5] = crc8(&buffer[3..5]);
                6
            }
            Transaction::Serial => {
                if buffer.len() < 6 {
                    return 0;
                }
                let serial = self.serial.get();
                buffer[0] = (serial >> 24) as u8;
                buffer[1] = (serial >> 16) as u8;
                buffer[2] = crc8(&buffer[0..2]);
                buffer[3] = (serial >> 8) as u8;
                buffer[4] = serial as u8;
                buffer[5] = crc8(&buffer[3..5]);
                6
            }
            Transaction::None => 0,
        }
    }

    // ---- Shared transaction handling, independent of which HIL trait the
    // ---- driver reached us through.

    fn perform_write(
        &self,
        data: &'static mut [u8],
        write_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        if self.buffer.is_some() {
            return Err((i2c::Error::Busy, data));
        }

        if write_len >= 1 {
            self.handle_command(data[0]);
        }
        self.finish(data, Ok(()));
        Ok(())
    }

    fn perform_read(
        &self,
        buffer: &'static mut [u8],
        read_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        if self.buffer.is_some() {
            return Err((i2c::Error::Busy, buffer));
        }

        // Pretend to be the chip clocking out `read_len` bytes of response.
        let response_len = read_len.min(buffer.len());
        let produced = self.fill_response(&mut buffer[..response_len]);
        let status = self.read_status(produced, read_len);
        self.pending.set(Transaction::None);
        self.finish(buffer, status);
        Ok(())
    }

    fn perform_write_read(
        &self,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        if self.buffer.is_some() {
            return Err((i2c::Error::Busy, data));
        }

        if write_len >= 1 {
            self.handle_command(data[0]);
        }
        let response_len = read_len.min(data.len());
        let produced = self.fill_response(&mut data[..response_len]);
        let status = self.read_status(produced, read_len);
        self.pending.set(Transaction::None);
        self.finish(data, status);
        Ok(())
    }

    /// Work out the completion status for a read of `requested` bytes when the
    /// mock had `produced` bytes of response to give.
    fn read_status(&self, produced: usize, requested: usize) -> Result<(), i2c::Error> {
        if produced >= requested {
            Ok(())
        } else if self.pending.get() == Transaction::None {
            // Nothing to read: a real chip would NAK the read.
            Err(i2c::Error::DataNak)
        } else {
            // Response is shorter than the driver asked for.
            Err(i2c::Error::Overrun)
        }
    }
}

impl<'a> crate::mock::i2c_bus::MockI2CDevice<'a> for MockSht4x<'a> {
    fn set_i2c_client(&'a self, client: &'a dyn i2c::I2CClient) {
        self.set_client(client);
    }
}

impl i2c::I2CDevice for MockSht4x<'_> {
    fn enable(&self) {}

    fn disable(&self) {}

    fn write(
        &self,
        data: &'static mut [u8],
        write_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        self.perform_write(data, write_len)
    }

    fn read(
        &self,
        buffer: &'static mut [u8],
        read_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        self.perform_read(buffer, read_len)
    }

    fn write_read(
        &self,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        self.perform_write_read(data, write_len, read_len)
    }
}

impl DeferredCallClient for MockSht4x<'_> {
    fn handle_deferred_call(&self) {
        // Stand-in for the "I2C transaction complete" interrupt: hand the
        // buffer back to whichever client is wired up, with the transaction's
        // result.
        let status = self.status.get();
        if let Some(buffer) = self.buffer.take() {
            if let Some(master_client) = self.master_client.get() {
                master_client.command_complete(buffer, status);
            } else if let Some(client) = self.client.get() {
                client.command_complete(buffer, status);
            } else {
                // No client registered: keep the buffer so it isn't lost.
                self.buffer.replace(buffer);
            }
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
