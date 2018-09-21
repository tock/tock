//! Driver for the LTC294X line of coulomb counters.
//!
//! - <http://www.linear.com/product/LTC2941>
//! - <http://www.linear.com/product/LTC2942>
//! - <http://www.linear.com/product/LTC2943>
//!
//! > The LTC2941 measures battery charge state in battery-supplied handheld PC
//! > and portable product applications. Its operating range is perfectly suited
//! > for single-cell Li-Ion batteries. A precision coulomb counter integrates
//! > current through a sense resistor between the battery’s positive terminal
//! > and the load or charger. The measured charge is stored in internal
//! > registers. An SMBus/I2C interface accesses and configures the device.
//!
//! Structure
//! ---------
//!
//! This file implements the LTC294X driver in two objects. First is the
//! `LTC294X` struct. This implements all of the actual logic for the
//! chip. The second is the `LTC294XDriver` struct. This implements the
//! userland facing syscall interface. These are split to allow the kernel
//! to potentially interface with the LTC294X chip rather than only provide
//! it to userspace.
//!
//! Usage
//! -----
//!
//! Here is a sample usage of this capsule in a board's main.rs file:
//!
//! ```rust
//! let ltc294x_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_mux, 0x64));
//! let ltc294x = static_init!(
//!     capsules::ltc294x::LTC294X<'static>,
//!     capsules::ltc294x::LTC294X::new(ltc294x_i2c, None, &mut capsules::ltc294x::BUFFER));
//! ltc294x_i2c.set_client(ltc294x);
//!
//! // Optionally create the object that provides an interface for the coulomb
//! // counter for applications.
//! let ltc294x_driver = static_init!(
//!     capsules::ltc294x::LTC294XDriver<'static>,
//!     capsules::ltc294x::LTC294XDriver::new(ltc294x));
//! ltc294x.set_client(ltc294x_driver);
//! ```

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::gpio;
use kernel::hil::i2c;
use kernel::ReturnCode;
use kernel::{AppId, Callback, Driver};

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x80000;

pub static mut BUFFER: [u8; 20] = [0; 20];

#[allow(dead_code)]
enum Registers {
    Status = 0x00,
    Control = 0x01,
    AccumulatedChargeMSB = 0x02,
    AccumulatedChargeLSB = 0x03,
    ChargeThresholdHighMSB = 0x04,
    ChargeThresholdHighLSB = 0x05,
    ChargeThresholdLowMSB = 0x06,
    ChargeThresholdLowLSB = 0x07,
    VoltageMSB = 0x08,
    VoltageLSB = 0x09,
    CurrentMSB = 0x0E,
    CurrentLSB = 0x0F,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,

    /// Simple read states
    ReadStatus,
    ReadCharge,
    ReadVoltage,
    ReadCurrent,
    ReadShutdown,

    Done,
}

/// Which version of the chip we are actually using.
#[derive(Clone, Copy)]
pub enum ChipModel {
    LTC2941 = 1,
    LTC2942 = 2,
    LTC2943 = 3,
}

/// Settings for which interrupt we want.
pub enum InterruptPinConf {
    Disabled = 0x00,
    ChargeCompleteMode = 0x01,
    AlertMode = 0x02,
}

/// Threshold options for battery alerts.
pub enum VBatAlert {
    Off = 0x00,
    Threshold2V8 = 0x01,
    Threshold2V9 = 0x02,
    Threshold3V0 = 0x03,
}

/// Supported events for the LTC294X.
pub trait LTC294XClient {
    fn interrupt(&self);
    fn status(
        &self,
        undervolt_lockout: bool,
        vbat_alert: bool,
        charge_alert_low: bool,
        charge_alert_high: bool,
        accumulated_charge_overflow: bool,
    );
    fn charge(&self, charge: u16);
    fn voltage(&self, voltage: u16);
    fn current(&self, current: u16);
    fn done(&self);
}

/// Implementation of a driver for the LTC294X coulomb counters.
pub struct LTC294X<'a> {
    i2c: &'a i2c::I2CDevice,
    interrupt_pin: Option<&'a gpio::Pin>,
    model: Cell<ChipModel>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    client: OptionalCell<&'static LTC294XClient>,
}

impl LTC294X<'a> {
    pub fn new(
        i2c: &'a i2c::I2CDevice,
        interrupt_pin: Option<&'a gpio::Pin>,
        buffer: &'static mut [u8],
    ) -> LTC294X<'a> {
        LTC294X {
            i2c: i2c,
            interrupt_pin: interrupt_pin,
            model: Cell::new(ChipModel::LTC2941),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client<C: LTC294XClient>(&self, client: &'static C) {
        self.client.set(client);

        self.interrupt_pin.map(|interrupt_pin| {
            interrupt_pin.make_input();
            interrupt_pin.enable_interrupt(0, gpio::InterruptMode::FallingEdge);
        });
    }

    pub fn read_status(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c.enable();

            // Address pointer automatically resets to the status register.
            self.i2c.read(buffer, 1);
            self.state.set(State::ReadStatus);

            ReturnCode::SUCCESS
        })
    }

    fn configure(
        &self,
        int_pin_conf: InterruptPinConf,
        prescaler: u8,
        vbat_alert: VBatAlert,
    ) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c.enable();

            buffer[0] = Registers::Control as u8;
            buffer[1] = ((int_pin_conf as u8) << 1) | (prescaler << 3) | ((vbat_alert as u8) << 6);

            self.i2c.write(buffer, 2);
            self.state.set(State::Done);

            ReturnCode::SUCCESS
        })
    }

    /// Set the accumulated charge to 0
    fn reset_charge(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c.enable();

            buffer[0] = Registers::AccumulatedChargeMSB as u8;
            buffer[1] = 0;
            buffer[2] = 0;

            self.i2c.write(buffer, 3);
            self.state.set(State::Done);

            ReturnCode::SUCCESS
        })
    }

    fn set_high_threshold(&self, threshold: u16) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c.enable();

            buffer[0] = Registers::ChargeThresholdHighMSB as u8;
            buffer[1] = ((threshold & 0xFF00) >> 8) as u8;
            buffer[2] = (threshold & 0xFF) as u8;

            self.i2c.write(buffer, 3);
            self.state.set(State::Done);

            ReturnCode::SUCCESS
        })
    }

    fn set_low_threshold(&self, threshold: u16) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c.enable();

            buffer[0] = Registers::ChargeThresholdLowMSB as u8;
            buffer[1] = ((threshold & 0xFF00) >> 8) as u8;
            buffer[2] = (threshold & 0xFF) as u8;

            self.i2c.write(buffer, 3);
            self.state.set(State::Done);

            ReturnCode::SUCCESS
        })
    }

    /// Get the cumulative charge as measured by the LTC2941.
    fn get_charge(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c.enable();

            // Read all of the first four registers rather than wasting
            // time writing an address.
            self.i2c.read(buffer, 4);
            self.state.set(State::ReadCharge);

            ReturnCode::SUCCESS
        })
    }

    /// Get the voltage at sense+
    fn get_voltage(&self) -> ReturnCode {
        // Not supported on all versions
        match self.model.get() {
            ChipModel::LTC2942 |
            ChipModel::LTC2943 => {
                self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
                    self.i2c.enable();

                    self.i2c.read(buffer, 10);
                    self.state.set(State::ReadVoltage);

                    ReturnCode::SUCCESS
                })
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Get the current sensed by the resistor
    fn get_current(&self) -> ReturnCode {
        // Not supported on all versions
        match self.model.get() {
            ChipModel::LTC2943 => {
                self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
                    self.i2c.enable();

                    self.i2c.read(buffer, 16);
                    self.state.set(State::ReadCurrent);

                    ReturnCode::SUCCESS
                })
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Put the LTC294X in a low power state.
    fn shutdown(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c.enable();

            // Read both the status and control register rather than
            // writing an address.
            self.i2c.read(buffer, 2);
            self.state.set(State::ReadShutdown);

            ReturnCode::SUCCESS
        })
    }

    /// Set the LTC294X model actually on the board.
    fn set_model(&self, model_num: usize) -> ReturnCode {
        match model_num {
            1 => {
                self.model.set(ChipModel::LTC2941);
                ReturnCode::SUCCESS
            }
            2 => {
                self.model.set(ChipModel::LTC2942);
                ReturnCode::SUCCESS
            }
            3 => {
                self.model.set(ChipModel::LTC2943);
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENODEVICE,
        }
    }
}

impl i2c::I2CClient for LTC294X<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        match self.state.get() {
            State::ReadStatus => {
                let status = buffer[0];
                let uvlock = (status & 0x01) > 0;
                let vbata = (status & 0x02) > 0;
                let ca_low = (status & 0x04) > 0;
                let ca_high = (status & 0x08) > 0;
                let accover = (status & 0x20) > 0;
                self.client.map(|client| {
                    client.status(uvlock, vbata, ca_low, ca_high, accover);
                });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::ReadCharge => {
                // Charge is calculated in user space
                let charge = ((buffer[2] as u16) << 8) | (buffer[3] as u16);
                self.client.map(|client| { client.charge(charge); });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::ReadVoltage => {
                let voltage = ((buffer[8] as u16) << 8) | (buffer[9] as u16);
                self.client.map(|client| { client.voltage(voltage); });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::ReadCurrent => {
                let current = ((buffer[14] as u16) << 8) | (buffer[15] as u16);
                self.client.map(|client| { client.current(current); });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::ReadShutdown => {
                // Set the shutdown pin to 1
                buffer[1] |= 0x01;

                // Write the control register back but with a 1 in the shutdown
                // bit.
                buffer[0] = Registers::Control as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::Done);
            }
            State::Done => {
                self.client.map(|client| { client.done(); });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            _ => {}
        }
    }
}

impl gpio::Client for LTC294X<'a> {
    fn fired(&self, _: usize) {
        self.client.map(|client| { client.interrupt(); });
    }
}

/// Default implementation of the LTC2941 driver that provides a Driver
/// interface for providing access to applications.
pub struct LTC294XDriver<'a> {
    ltc294x: &'a LTC294X<'a>,
    callback: OptionalCell<Callback>,
}

impl LTC294XDriver<'a> {
    pub fn new(ltc: &'a LTC294X) -> LTC294XDriver<'a> {
        LTC294XDriver {
            ltc294x: ltc,
            callback: OptionalCell::empty(),
        }
    }
}

impl LTC294XClient for LTC294XDriver<'a> {
    fn interrupt(&self) {
        self.callback.map(|cb| { cb.schedule(0, 0, 0); });
    }

    fn status(
        &self,
        undervolt_lockout: bool,
        vbat_alert: bool,
        charge_alert_low: bool,
        charge_alert_high: bool,
        accumulated_charge_overflow: bool,
    ) {
        self.callback.map(|cb| {
            let ret = (undervolt_lockout as usize) | ((vbat_alert as usize) << 1) |
                ((charge_alert_low as usize) << 2) |
                ((charge_alert_high as usize) << 3) |
                ((accumulated_charge_overflow as usize) << 4);
            cb.schedule(1, ret, self.ltc294x.model.get() as usize);
        });
    }

    fn charge(&self, charge: u16) {
        self.callback.map(
            |cb| { cb.schedule(2, charge as usize, 0); },
        );
    }

    fn done(&self) {
        self.callback.map(|cb| { cb.schedule(3, 0, 0); });
    }

    fn voltage(&self, voltage: u16) {
        self.callback.map(
            |cb| { cb.schedule(4, voltage as usize, 0); },
        );
    }

    fn current(&self, current: u16) {
        self.callback.map(
            |cb| { cb.schedule(5, current as usize, 0); },
        );
    }
}

impl Driver for LTC294XDriver<'a> {
    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Set the callback that that is triggered when events finish and
    ///   when readings are ready. The first argument represents which callback
    ///   was triggered.
    ///   - `0`: Interrupt occurred from the LTC294X.
    ///   - `1`: Got the status.
    ///   - `2`: Read the charge used.
    ///   - `3`: `done()` was called.
    ///   - `4`: Read the voltage.
    ///   - `5`: Read the current.
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => {
                self.callback.insert(callback);
                ReturnCode::SUCCESS
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Request operations for the LTC294X chip.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Get status of the chip.
    /// - `2`: Configure settings of the chip.
    /// - `3`: Reset accumulated charge measurement to zero.
    /// - `4`: Set the upper threshold for charge.
    /// - `5`: Set the lower threshold for charge.
    /// - `6`: Get the current charge accumulated.
    /// - `7`: Shutdown the chip.
    /// - `8`: Get the voltage reading. Only supported on the LTC2942 and
    ///   LTC2943.
    /// - `9`: Get the current reading. Only supported on the LTC2943.
    /// - `10`: Set the model of the LTC294X actually being used. `data` is the
    ///   value of the X.
    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            // Check this driver exists.
            0 => ReturnCode::SUCCESS,

            // Get status.
            1 => self.ltc294x.read_status(),

            // Configure.
            2 => {
                let int_pin_raw = data & 0x03;
                let prescaler = (data >> 2) & 0x07;
                let vbat_raw = (data >> 5) & 0x03;
                let int_pin_conf = match int_pin_raw {
                    0 => InterruptPinConf::Disabled,
                    1 => InterruptPinConf::ChargeCompleteMode,
                    2 => InterruptPinConf::AlertMode,
                    _ => InterruptPinConf::Disabled,
                };
                let vbat_alert = match vbat_raw {
                    0 => VBatAlert::Off,
                    1 => VBatAlert::Threshold2V8,
                    2 => VBatAlert::Threshold2V9,
                    3 => VBatAlert::Threshold3V0,
                    _ => VBatAlert::Off,
                };

                self.ltc294x.configure(
                    int_pin_conf,
                    prescaler as u8,
                    vbat_alert,
                )
            }

            // Reset charge.
            3 => self.ltc294x.reset_charge(),

            // Set high threshold
            4 => self.ltc294x.set_high_threshold(data as u16),

            // Set low threshold
            5 => self.ltc294x.set_low_threshold(data as u16),

            // Get charge
            6 => self.ltc294x.get_charge(),

            // Shutdown
            7 => self.ltc294x.shutdown(),

            // Get voltage
            8 => self.ltc294x.get_voltage(),

            // Get current
            9 => self.ltc294x.get_current(),

            // Set the current chip model
            10 => self.ltc294x.set_model(data),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
