//! Driver for the Taos TSL2561 light sensor.
//!
//! <http://www.digikey.com/product-detail/en/ams-taos-usa-inc/TSL2561FN/TSL2561-FNCT-ND/3095298>
//!
//! > The TSL2560 and TSL2561 are light-to-digital converters that transform
//! > light intensity to a digital signal output capable of direct I2C
//! > interface. Each device combines one broadband photodiode (visible plus
//! > infrared) and one infrared-responding photodiode on a single CMOS
//! > integrated circuit capable of providing a near-photopic response over an
//! > effective 20-bit dynamic range (16-bit resolution). Two integrating ADCs
//! > convert the photodiode currents to a digital output that represents the
//! > irradiance measured on each channel. This digital output can be input to a
//! > microprocessor where illuminance (ambient light level) in lux is derived
//! > using an empirical formula to approximate the human eye response.

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::gpio;
use kernel::hil::i2c;
use kernel::{AppId, Callback, LegacyDriver, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Tsl2561 as usize;

// Buffer to use for I2C messages
pub static mut BUFFER: [u8; 4] = [0; 4];

/// Command register defines
const COMMAND_REG: u8 = 0x80;
const WORD_PROTOCOL: u8 = 0x20;

/// Control_Reg defines
const POWER_ON: u8 = 0x03;
const POWER_OFF: u8 = 0x00;

/// Timing_Reg defines
const INTEGRATE_TIME_101_MS: u8 = 0x01;
const LOW_GAIN_MODE: u8 = 0x00;

// Interrupt_Control_Reg defines
const INTERRUPT_CONTROL_LEVEL: u8 = 0x10;
const INTERRUPT_ON_ADC_DONE: u8 = 0x0;

// ADC counts to Lux value conversion copied from TSL2561 manual
// −−−−------------------------------
// Value scaling factors
// −−−−−−−−−−−−−−−-------------------
const LUX_SCALE: u16 = 14; // scale by 2^14
const RATIO_SCALE: u16 = 9; // scale ratio by 2^9

// −−−−−−−−−−−−−−−−−−−−−−−-----------
// Integration time scaling factors
// −−−−−−−−−−−−−−−−−−−−−−−−−−−−------
const CH_SCALE: u16 = 10; // scale channel values by 2^10
#[allow(dead_code)]
const CHSCALE_TINT0: u16 = 0x7517; // 322/11 * 2^CH_SCALE
const CHSCALE_TINT1: u16 = 0x0fe7; // 322/81 * 2^CH_SCALE

// −−−−−−−−−−−−−−−−−−−−−−−−−−−−------
// T, FN, and CL Package coefficients
// −−−−−−−−−−−−−−−−−−−−−−−−−−−−------
// For Ch1/Ch0=0.00 to 0.50
// Lux/Ch0=0.0304−0.062*((Ch1/Ch0)^1.4)
// piecewise approximation
// For Ch1/Ch0=0.00 to 0.125:
// Lux/Ch0=0.0304−0.0272*(Ch1/Ch0)
//
// For Ch1/Ch0=0.125 to 0.250:
// Lux/Ch0=0.0325−0.0440*(Ch1/Ch0)
//
// For Ch1/Ch0=0.250 to 0.375:
// Lux/Ch0=0.0351−0.0544*(Ch1/Ch0)
//
// For Ch1/Ch0=0.375 to 0.50:
// Lux/Ch0=0.0381−0.0624*(Ch1/Ch0)
//
// For Ch1/Ch0=0.50 to 0.61:
// Lux/Ch0=0.0224−0.031*(Ch1/Ch0)
//
// For Ch1/Ch0=0.61 to 0.80:
// Lux/Ch0=0.0128−0.0153*(Ch1/Ch0)
//
// For Ch1/Ch0=0.80 to 1.30:
// Lux/Ch0=0.00146−0.00112*(Ch1/Ch0)
//
// For Ch1/Ch0>1.3:
// Lux/Ch0=0
// −−−−−−−−−−−−−−−−−−−−−−−−−−−−------
const K1T: usize = 0x0040; // 0.125 * 2^RATIO_SCALE
const B1T: usize = 0x01f2; // 0.0304 * 2^LUX_SCALE
const M1T: usize = 0x01be; // 0.0272 * 2^LUX_SCALE
const K2T: usize = 0x0080; // 0.250 * 2^RATIO_SCALE
const B2T: usize = 0x0214; // 0.0325 * 2^LUX_SCALE
const M2T: usize = 0x02d1; // 0.0440 * 2^LUX_SCALE
const K3T: usize = 0x00c0; // 0.375 * 2^RATIO_SCALE
const B3T: usize = 0x023f; // 0.0351 * 2^LUX_SCALE
const M3T: usize = 0x037b; // 0.0544 * 2^LUX_SCALE
const K4T: usize = 0x0100; // 0.50 * 2^RATIO_SCALE
const B4T: usize = 0x0270; // 0.0381 * 2^LUX_SCALE
const M4T: usize = 0x03fe; // 0.0624 * 2^LUX_SCALE
const K5T: usize = 0x0138; // 0.61 * 2^RATIO_SCALE
const B5T: usize = 0x016f; // 0.0224 * 2^LUX_SCALE
const M5T: usize = 0x01fc; // 0.0310 * 2^LUX_SCALE
const K6T: usize = 0x019a; // 0.80 * 2^RATIO_SCALE
const B6T: usize = 0x00d2; // 0.0128 * 2^LUX_SCALE
const M6T: usize = 0x00fb; // 0.0153 * 2^LUX_SCALE
const K7T: usize = 0x029a; // 1.3 * 2^RATIO_SCALE
const B7T: usize = 0x0018; // 0.00146 * 2^LUX_SCALE
const M7T: usize = 0x0012; // 0.00112 * 2^LUX_SCALE
const K8T: usize = 0x029a; // 1.3 * 2^RATIO_SCALE
const B8T: usize = 0x0000; // 0.000 * 2^LUX_SCALE
const M8T: usize = 0x0000; // 0.000 * 2^LUX_SCALE

// −−−−−−−−−−−−−−−−−−−−−−−−−−−−------
// CS package coefficients
// −−−−−−−−−−−−−−−−−−−−−−−−−−−−------
// For 0 <= Ch1/Ch0 <= 0.52
// Lux/Ch0 = 0.0315−0.0593*((Ch1/Ch0)^1.4)
// piecewise approximation
// For 0 <= Ch1/Ch0 <= 0.13
// Lux/Ch0 = 0.0315−0.0262*(Ch1/Ch0)
// For 0.13 <= Ch1/Ch0 <= 0.26
// Lux/Ch0 = 0.0337−0.0430*(Ch1/Ch0)
// For 0.26 <= Ch1/Ch0 <= 0.39
// Lux/Ch0 = 0.0363−0.0529*(Ch1/Ch0)
// For 0.39 <= Ch1/Ch0 <= 0.52
// Lux/Ch0 = 0.0392−0.0605*(Ch1/Ch0)
// For 0.52 < Ch1/Ch0 <= 0.65
// Lux/Ch0 = 0.0229−0.0291*(Ch1/Ch0)
// For 0.65 < Ch1/Ch0 <= 0.80
// Lux/Ch0 = 0.00157−0.00180*(Ch1/Ch0)
// For 0.80 < Ch1/Ch0 <= 1.30
// Lux/Ch0 = 0.00338−0.00260*(Ch1/Ch0)
// For Ch1/Ch0 > 1.30
// Lux = 0
// −−−−−−−−−−−−−−−−−−−−−−−−−−−−------
// const K1C: usize = 0x0043; // 0.130 * 2^RATIO_SCALE
// const B1C: usize = 0x0204; // 0.0315 * 2^LUX_SCALE
// const M1C: usize = 0x01ad; // 0.0262 * 2^LUX_SCALE
// const K2C: usize = 0x0085; // 0.260 * 2^RATIO_SCALE
// const B2C: usize = 0x0228; // 0.0337 * 2^LUX_SCALE
// const M2C: usize = 0x02c1; // 0.0430 * 2^LUX_SCALE
// const K3C: usize = 0x00c8; // 0.390 * 2^RATIO_SCALE
// const B3C: usize = 0x0253; // 0.0363 * 2^LUX_SCALE
// const M3C: usize = 0x0363; // 0.0529 * 2^LUX_SCALE
// const K4C: usize = 0x010a; // 0.520 * 2^RATIO_SCALE
// const B4C: usize = 0x0282; // 0.0392 * 2^LUX_SCALE
// const M4C: usize = 0x03df; // 0.0605 * 2^LUX_SCALE
// const K5C: usize = 0x014d; // 0.65 * 2^RATIO_SCALE
// const B5C: usize = 0x0177; // 0.0229 * 2^LUX_SCALE
// const M5C: usize = 0x01dd; // 0.0291 * 2^LUX_SCALE
// const K6C: usize = 0x019a; // 0.80 * 2^RATIO_SCALE
// const B6C: usize = 0x0101; // 0.0157 * 2^LUX_SCALE
// const M6C: usize = 0x0127; // 0.0180 * 2^LUX_SCALE
// const K7C: usize = 0x029a; // 1.3 * 2^RATIO_SCALE
// const B7C: usize = 0x0037; // 0.00338 * 2^LUX_SCALE
// const M7C: usize = 0x002b; // 0.00260 * 2^LUX_SCALE
// const K8C: usize = 0x029a; // 1.3 * 2^RATIO_SCALE
// const B8C: usize = 0x0000; // 0.000 * 2^LUX_SCALE
// const M8C: usize = 0x0000; // 0.000 * 2^LUX_SCALE

#[allow(dead_code)]
enum Registers {
    Control = 0x00,
    Timing = 0x01,
    ThresholdLowLow = 0x02,
    ThresholdLowHigh = 0x03,
    ThresholdHighLow = 0x04,
    ThresholdHighHigh = 0x05,
    Interrupt = 0x06,
    Id = 0x0a,
    Data0Low = 0x0c,
    Data0High = 0x0d,
    Data1Low = 0x0e,
    Data1High = 0x0f,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,

    /// Read the Id register.
    SelectId,
    ReadingId,

    /// Process of taking a light measurement.
    TakeMeasurementTurnOn,
    TakeMeasurementConfigMeasurement,
    TakeMeasurementReset1,
    TakeMeasurementReset2,

    /// Read the ADC registers.
    ReadMeasurement1,
    ReadMeasurement2,
    ReadMeasurement3,
    /// Calculate light and call the callback with the value.
    GotMeasurement,

    /// Disable I2C and release buffer
    Done,
}

pub struct TSL2561<'a> {
    i2c: &'a dyn i2c::I2CDevice,
    interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
    callback: OptionalCell<Callback>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a> TSL2561<'a> {
    pub fn new(
        i2c: &'a dyn i2c::I2CDevice,
        interrupt_pin: &'a dyn gpio::InterruptPin<'a>,
        buffer: &'static mut [u8],
    ) -> TSL2561<'a> {
        // setup and return struct
        TSL2561 {
            i2c: i2c,
            interrupt_pin: interrupt_pin,
            callback: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
        }
    }

    pub fn read_id(&self) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            buffer[0] = Registers::Id as u8 | COMMAND_REG;
            // buffer[0] = Registers::Id as u8;
            self.i2c.write(buffer, 1);
            self.state.set(State::SelectId);
        });
    }

    pub fn take_measurement(&self) {
        // Need pull up on interrupt pin
        self.interrupt_pin.make_input();
        self.interrupt_pin
            .enable_interrupts(gpio::InterruptEdge::FallingEdge);

        self.buffer.take().map(|buf| {
            // Turn on i2c to send commands
            self.i2c.enable();

            buf[0] = Registers::Control as u8 | COMMAND_REG;
            buf[1] = POWER_ON;
            self.i2c.write(buf, 2);
            self.state.set(State::TakeMeasurementTurnOn);
        });
    }

    fn calculate_lux(&self, chan0: u16, chan1: u16) -> usize {
        // First, scale the channel values depending on the gain and integration
        // time. 16X, 402mS is nominal. Scale if integration time is NOT 402 msec.
        // let mut ch_scale = CHSCALE_TINT0 as usize; // 13.7ms
        let mut ch_scale = CHSCALE_TINT1 as usize; // 101ms
                                                   // let mut ch_scale: usize = 1 << CH_SCALE; // Default

        // Scale if gain is NOT 16X
        ch_scale = ch_scale << 4; // scale 1X to 16X

        // scale the channel values
        let channel0 = (chan0 as usize * ch_scale) >> CH_SCALE;
        let channel1 = (chan1 as usize * ch_scale) >> CH_SCALE;

        // Find the ratio of the channel values (Channel1/Channel0).
        // Protect against divide by zero.
        let mut ratio1 = 0;
        if channel0 != 0 {
            ratio1 = (channel1 << (RATIO_SCALE + 1)) / channel0;
        }

        // round the ratio value
        let ratio = (ratio1 + 1) >> 1;

        // is ratio <= eachBreak ?
        let mut b = 0;
        let mut m = 0;
        // T, FN, and CL package
        if ratio <= K1T {
            b = B1T;
            m = M1T;
        } else if ratio <= K2T {
            b = B2T;
            m = M2T;
        } else if ratio <= K3T {
            b = B3T;
            m = M3T;
        } else if ratio <= K4T {
            b = B4T;
            m = M4T;
        } else if ratio <= K5T {
            b = B5T;
            m = M5T;
        } else if ratio <= K6T {
            b = B6T;
            m = M6T;
        } else if ratio <= K7T {
            b = B7T;
            m = M7T;
        } else if ratio > K8T {
            b = B8T;
            m = M8T;
        }
        // CS package
        // if ratio <= K1C {
        //     b=B1C; m=M1C;
        // } else if ratio <= K2C {
        //     b=B2C; m=M2C;
        // } else if ratio <= K3C {
        //     b=B3C; m=M3C;
        // } else if ratio <= K4C {
        //     b=B4C; m=M4C;
        // } else if ratio <= K5C {
        //     b=B5C; m=M5C;
        // } else if ratio <= K6C {
        //     b=B6C; m=M6C;
        // } else if ratio <= K7C {
        //     b=B7C; m=M7C;
        // } else if ratio > K8C {
        //     b=B8C; m=M8C;
        // }

        // Calculate actual lux value
        let mut val = ((channel0 * b) as isize) - ((channel1 * m) as isize);

        // Do not allow negative lux value
        if val < 0 {
            val = 0;
        }

        // round lsb (2^(LUX_SCALE−1))
        // val += (1 << (LUX_SCALE−1));
        val += 1 << (LUX_SCALE - 1);

        // strip off fractional portion and return lux
        let lux = val >> LUX_SCALE;

        lux as usize
    }
}

impl i2c::I2CClient for TSL2561<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        match self.state.get() {
            State::SelectId => {
                self.i2c.read(buffer, 1);
                self.state.set(State::ReadingId);
            }
            State::ReadingId => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::TakeMeasurementTurnOn => {
                buffer[0] = Registers::Timing as u8 | COMMAND_REG;
                buffer[1] = INTEGRATE_TIME_101_MS | LOW_GAIN_MODE;
                self.i2c.write(buffer, 2);
                self.state.set(State::TakeMeasurementConfigMeasurement);
            }
            State::TakeMeasurementConfigMeasurement => {
                buffer[0] = Registers::Interrupt as u8 | COMMAND_REG;
                buffer[1] = INTERRUPT_CONTROL_LEVEL | INTERRUPT_ON_ADC_DONE;
                self.i2c.write(buffer, 2);
                self.state.set(State::TakeMeasurementReset1);
            }
            State::TakeMeasurementReset1 => {
                buffer[0] = Registers::Control as u8 | COMMAND_REG;
                buffer[1] = POWER_OFF;
                self.i2c.write(buffer, 2);
                self.state.set(State::TakeMeasurementReset2);
            }
            State::TakeMeasurementReset2 => {
                buffer[0] = Registers::Control as u8 | COMMAND_REG;
                buffer[1] = POWER_ON;
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            }
            State::ReadMeasurement1 => {
                self.i2c.read(buffer, 2);
                self.state.set(State::ReadMeasurement2);
            }
            State::ReadMeasurement2 => {
                // Store the previous readings in the buffer where they
                // won't get overwritten.
                buffer[2] = buffer[0];
                buffer[3] = buffer[1];
                buffer[0] = Registers::Data0Low as u8 | COMMAND_REG | WORD_PROTOCOL;
                self.i2c.write(buffer, 2);
                self.state.set(State::ReadMeasurement3);
            }
            State::ReadMeasurement3 => {
                self.i2c.read(buffer, 2);
                self.state.set(State::GotMeasurement);
            }
            State::GotMeasurement => {
                let chan0 = ((buffer[1] as u16) << 8) | (buffer[0] as u16);
                let chan1 = ((buffer[3] as u16) << 8) | (buffer[2] as u16);

                let lux = self.calculate_lux(chan0, chan1);

                self.callback.map(|cb| cb.schedule(0, lux, 0));

                buffer[0] = Registers::Control as u8 | COMMAND_REG;
                buffer[1] = POWER_OFF;
                self.i2c.write(buffer, 2);
                self.interrupt_pin.disable_interrupts();
                self.state.set(State::Done);
            }
            State::Done => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            _ => {}
        }
    }
}

impl gpio::Client for TSL2561<'_> {
    fn fired(&self) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            // Read the first of the ADC registers.
            buffer[0] = Registers::Data1Low as u8 | COMMAND_REG | WORD_PROTOCOL;
            self.i2c.write(buffer, 1);
            self.state.set(State::ReadMeasurement1);
        });
    }
}

impl LegacyDriver for TSL2561<'_> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // Set a callback
            0 => {
                // Set callback function
                self.callback.insert(callback);
                ReturnCode::SUCCESS
            }
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, _: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            // Take a measurement
            1 => {
                self.take_measurement();
                ReturnCode::SUCCESS
            }
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
