// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Mock implementation of the Bosch BME280 humidity/pressure/temperature
//! sensor.
//!
//! Following the same approach as [`crate::mock::sensors::sht4x`], this capsule
//! pretends to be a real BME280 chip on an I2C bus so the [`crate::bme280`]
//! driver (and the pressure/temperature/humidity syscall drivers on top of it)
//! can be exercised with no physical bus and no physical sensor.
//!
//! It implements the [`kernel::hil::i2c::I2CDevice`] HIL and, via
//! [`MockI2CDevice`](crate::mock::i2c_bus::MockI2CDevice), can be attached to a
//! [`MockI2CBus`](crate::mock::i2c_bus::MockI2CBus). There is no real bus and
//! no real interrupt: the mock uses a [`DeferredCall`] to asynchronously invoke
//! the client's `command_complete`.
//!
//! The mock behaves like the chip: it inspects the register address the driver
//! writes, and answers reads with the chip ID, a fixed set of factory
//! calibration words, and synthesized raw ADC values. The raw ADC values are
//! chosen (by inverting / searching the exact fixed-point compensation the
//! driver applies, using the same calibration words the mock reports) so the
//! driver decodes back to the configured [`set_temperature`](MockBme280::set_temperature),
//! [`set_pressure`](MockBme280::set_pressure) and
//! [`set_humidity`](MockBme280::set_humidity) values.
//!
//! Because the BME280 pressure compensation depends on the temperature reading
//! (`t_fine`), the reported pressure is only exact once a temperature reading
//! has been taken; the mock assumes the driver's `t_fine` corresponds to the
//! configured temperature.

use core::cell::Cell;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::i2c;
use kernel::utilities::cells::{OptionalCell, TakeCell};

/// I2C bus address of the (real) BME280. Matches the address used by the
/// `bme280` board component examples.
pub const BASE_ADDR: u8 = 0x77;

// BME280 register addresses (the first byte the driver writes before a read).
const REG_ID: u8 = 0xD0;
const REG_CALIB00: u8 = 0x88;
const REG_CALIB26: u8 = 0xE1;
const REG_HUM_MSB: u8 = 0xFD;
const REG_TEMP_MSB: u8 = 0xFA;
const REG_PRESS_MSB: u8 = 0xF7;

/// Chip ID a real BME280 returns from [`REG_ID`]; the driver bails out unless
/// it sees exactly this.
const CHIP_ID: u8 = 0x60;

/// Raw 20-bit pressure value the driver treats as "measurement skipped".
const SKIPPED_PRESSURE_READING: i64 = 0x8_0000;

// Fixed factory calibration words the mock reports. These are realistic
// BME280 values, except `DIG_T3`, which is forced to a tiny non-zero value:
// the driver rejects (and retries forever on) zero temperature calibration,
// but a small value keeps the temperature inversion below a simple linear
// term over the whole operating range.
//
// Matching the `bme280` driver, T2/T3 are interpreted as unsigned and the
// pressure words as signed.
const DIG_T1: u16 = 27504;
const DIG_T2: u16 = 26435;
const DIG_T3: u16 = 1;
const DIG_P1: u16 = 36477;
const DIG_P2: i16 = -10685;
const DIG_P3: i16 = 3024;
const DIG_P4: i16 = 2855;
const DIG_P5: i16 = 140;
const DIG_P6: i16 = -7;
const DIG_P7: i16 = 15500;
const DIG_P8: i16 = -14600;
const DIG_P9: i16 = 6000;
const DIG_H1: u8 = 75;
const DIG_H2: i16 = 362;
const DIG_H3: u8 = 0;

/// Default temperature the mock reports, hundredths of a degree Celsius
/// (21.00 C).
const DEFAULT_TEMPERATURE_CENTI_C: i32 = 2100;
/// Default atmospheric pressure the mock reports, in hectopascals (hPa).
const DEFAULT_PRESSURE_HPA: u32 = 1013;
/// Default relative humidity the mock reports, hundredths of a percent
/// (50.00 %RH). (The BME280 humidity compensation is not inverted here, so
/// this is only a rough target.)
const DEFAULT_HUMIDITY_CENTI_PCT: u32 = 5000;

fn put_u16_le(buf: &mut [u8], value: u16) {
    buf[0] = value as u8;
    buf[1] = (value >> 8) as u8;
}

pub struct MockBme280<'a> {
    client: OptionalCell<&'a dyn i2c::I2CClient>,
    /// Buffer for the in-flight transaction, returned to the client when the
    /// deferred call fires.
    buffer: TakeCell<'static, [u8]>,
    /// Deferred call standing in for the "transaction complete" interrupt.
    deferred_call: DeferredCall,
    /// Result to report for the in-flight transaction.
    status: Cell<Result<(), i2c::Error>>,
    /// Reported temperature, hundredths of a degree Celsius.
    temperature: Cell<i32>,
    /// Reported atmospheric pressure, hectopascals.
    pressure: Cell<u32>,
    /// Reported relative humidity, hundredths of a percent.
    humidity: Cell<u32>,
}

impl Default for MockBme280<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MockBme280<'a> {
    pub fn new() -> Self {
        Self {
            client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            deferred_call: DeferredCall::new(),
            status: Cell::new(Ok(())),
            temperature: Cell::new(DEFAULT_TEMPERATURE_CENTI_C),
            pressure: Cell::new(DEFAULT_PRESSURE_HPA),
            humidity: Cell::new(DEFAULT_HUMIDITY_CENTI_PCT),
        }
    }

    /// Register the I2C completion-callback client.
    pub fn set_client(&'a self, client: &'a dyn i2c::I2CClient) {
        self.client.set(client);
    }

    /// Set the temperature the mock reports, hundredths of a degree Celsius
    /// (e.g. `2537` for 25.37 C).
    pub fn set_temperature(&self, centi_celsius: i32) {
        self.temperature.set(centi_celsius);
    }

    /// Set the atmospheric pressure the mock reports, in hectopascals.
    pub fn set_pressure(&self, hpa: u32) {
        self.pressure.set(hpa);
    }

    /// Set the relative humidity the mock reports, hundredths of a percent.
    pub fn set_humidity(&self, centi_percent: u32) {
        self.humidity.set(centi_percent);
    }

    /// Schedule the deferred call standing in for the transaction-complete
    /// interrupt.
    fn finish(&self, data: &'static mut [u8], status: Result<(), i2c::Error>) {
        self.buffer.replace(data);
        self.status.set(status);
        self.deferred_call.set();
    }

    /// Fill `buf` with the response a real BME280 would clock out for a read
    /// of register `reg`.
    fn fill_response(&self, reg: u8, buf: &mut [u8]) {
        match reg {
            REG_ID if !buf.is_empty() => buf[0] = CHIP_ID,
            REG_CALIB00 if buf.len() >= 26 => self.fill_calib_low(buf),
            REG_CALIB26 if buf.len() >= 8 => self.fill_calib_high(buf),
            REG_TEMP_MSB if buf.len() >= 3 => Self::fill_adc20(buf, self.adc_temperature()),
            REG_PRESS_MSB if buf.len() >= 3 => Self::fill_adc20(buf, self.adc_pressure()),
            REG_HUM_MSB if buf.len() >= 2 => {
                // Humidity compensation is not inverted; just return a
                // non-zero raw value so the driver doesn't treat it as a
                // misread.
                buf[0] = 0x80;
                buf[1] = 0x00;
            }
            _ => {}
        }
    }

    /// Calibration block at `0x88` (26 bytes), laid out as the driver parses
    /// it.
    fn fill_calib_low(&self, b: &mut [u8]) {
        put_u16_le(&mut b[0..2], DIG_T1);
        put_u16_le(&mut b[2..4], DIG_T2);
        put_u16_le(&mut b[4..6], DIG_T3);
        put_u16_le(&mut b[6..8], DIG_P1);
        put_u16_le(&mut b[8..10], DIG_P2 as u16);
        put_u16_le(&mut b[10..12], DIG_P3 as u16);
        put_u16_le(&mut b[12..14], DIG_P4 as u16);
        put_u16_le(&mut b[14..16], DIG_P5 as u16);
        put_u16_le(&mut b[16..18], DIG_P6 as u16);
        put_u16_le(&mut b[18..20], DIG_P7 as u16);
        put_u16_le(&mut b[20..22], DIG_P8 as u16);
        put_u16_le(&mut b[22..24], DIG_P9 as u16);
        b[24] = 0; // reserved register 0xA0
        b[25] = DIG_H1;
    }

    /// Calibration block at `0xE1` (8 bytes). Only the humidity words live
    /// here; they are left benign since humidity is not inverted.
    fn fill_calib_high(&self, b: &mut [u8]) {
        put_u16_le(&mut b[0..2], DIG_H2 as u16);
        b[2] = 0;
        b[3] = DIG_H3;
        b[4] = 0;
        b[5] = 0;
        b[6] = 0;
        b[7] = 0;
    }

    /// Pack a 20-bit ADC value into 3 bytes, MSB-first, as the driver unpacks
    /// it (`b[0] << 12 | b[1] << 4 | b[2] >> 4`).
    fn fill_adc20(b: &mut [u8], adc: i64) {
        let adc = adc.clamp(1, 0xF_FFFF);
        b[0] = (adc >> 12) as u8;
        b[1] = (adc >> 4) as u8;
        b[2] = ((adc << 4) & 0xF0) as u8;
    }

    /// The raw temperature ADC value that decodes to the configured
    /// temperature.
    ///
    /// Inverts `var1 = (((adc >> 3) - (T1 << 1)) * T2) >> 11` and
    /// `T = (t_fine * 5 + 128) >> 8`, ignoring the second-order `var2` term
    /// (negligible with `DIG_T3 == 1`).
    fn adc_temperature(&self) -> i64 {
        let t = self.temperature.get() as i64;
        let t_fine = ((t << 8) - 128) / 5;
        ((((t_fine << 11) / (DIG_T2 as i64)) + ((DIG_T1 as i64) << 1)) << 3).clamp(1, 0xF_FFFF)
    }

    /// The `t_fine` the driver computes from our raw temperature value, using
    /// the driver's exact fixed-point arithmetic.
    fn driver_t_fine(&self) -> i64 {
        let adc_t = self.adc_temperature();
        let t1 = DIG_T1 as i64;
        let var1 = (((adc_t >> 3) - (t1 << 1)) * (DIG_T2 as i64)) >> 11;
        let var2 = (((((adc_t >> 4) - t1) * ((adc_t >> 4) - t1)) >> 12) * (DIG_T3 as i64)) >> 14;
        var1 + var2
    }

    /// The raw pressure ADC value that decodes to the configured pressure.
    ///
    /// The driver's pressure compensation is monotonically decreasing in the
    /// raw value, so this binary-searches the 20-bit range using the exact
    /// same arithmetic.
    fn adc_pressure(&self) -> i64 {
        let target = self.pressure.get() as i64;
        let t_fine = self.driver_t_fine();

        let (mut lo, mut hi) = (0_i64, 0xF_FFFF_i64);
        while lo < hi {
            let mid = i64::midpoint(lo, hi);
            if Self::compensate_pressure(mid, t_fine) > target {
                // Pressure too high: need a larger raw value.
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }

        if lo == SKIPPED_PRESSURE_READING {
            lo + 1
        } else {
            lo
        }
    }

    /// The `bme280` driver's pressure compensation, verbatim, returning hPa.
    fn compensate_pressure(adc_p: i64, t_fine: i64) -> i64 {
        let (p1, p2, p3) = (DIG_P1 as i64, DIG_P2 as i64, DIG_P3 as i64);
        let (p4, p5, p6) = (DIG_P4 as i64, DIG_P5 as i64, DIG_P6 as i64);
        let (p7, p8, p9) = (DIG_P7 as i64, DIG_P8 as i64, DIG_P9 as i64);

        let mut var1 = t_fine - 128_000;
        let mut var2 = var1 * var1 * p6;
        var2 += (var1 * p5) << 17;
        var2 += p4 << 35;
        var1 = ((var1 * var1 * p3) >> 8) + ((var1 * p2) << 12);
        var1 = (((1_i64 << 47) + var1) * p1) >> 33;

        if var1 == 0 {
            return 0;
        }

        let mut p = 1_048_576 - adc_p;
        p = (((p << 31) - var2) * 3125) / var1;
        var1 = (p9 * (p >> 13) * (p >> 13)) >> 25;
        var2 = (p8 * p) >> 19;
        p = ((p + var1 + var2) >> 8) + (p7 << 4);

        // `p` is Q24.8 Pa; the driver returns hPa.
        p / 25_600
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

impl i2c::I2CDevice for MockBme280<'_> {
    fn enable(&self) {}

    fn disable(&self) {}

    fn write(
        &self,
        data: &'static mut [u8],
        _write_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        // Config-register writes (e.g. `CTRL_HUM`, `CTRL_MEAS`): just
        // acknowledge, no state to change in the mock.
        self.perform_transaction(data, None, 0)
    }

    fn read(
        &self,
        buffer: &'static mut [u8],
        _read_len: usize,
    ) -> Result<(), (i2c::Error, &'static mut [u8])> {
        // The `bme280` driver always uses `write_read`; a bare read has no
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

impl<'a> crate::mock::i2c_bus::MockI2CDevice<'a> for MockBme280<'a> {
    fn set_i2c_client(&'a self, client: &'a dyn i2c::I2CClient) {
        self.set_client(client);
    }
}

impl DeferredCallClient for MockBme280<'_> {
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
