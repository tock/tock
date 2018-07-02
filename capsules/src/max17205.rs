//! Driver for the Maxim MAX17205 fuel gauge.
//!
//! <https://www.maximintegrated.com/en/products/power/battery-management/MAX17205.html>
//!
//! > The MAX1720x/MAX1721x are ultra-low power stand-alone fuel gauge ICs that
//! > implement the Maxim ModelGauge™ m5 algorithm without requiring host
//! > interaction for configuration. This feature makes the MAX1720x/MAX1721x
//! > excellent pack-side fuel gauges. The MAX17201/MAX17211 monitor a single
//! > cell pack. The MAX17205/MAX17215 monitor and balance a 2S or 3S pack or
//! > monitor a multiple-series cell pack.
//!
//! Usage
//! -----
//!
//! ```rust
//! // Two i2c addresses are necessary.
//! // Registers 0x000-0x0FF are accessed by address 0x36.
//! // Registers 0x100-0x1FF are accessed by address 0x0B.
//! let max17205_i2c_lower = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x36));
//! let max17205_i2c_upper = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x0B));
//! let max17205 = static_init!(
//!     capsules::max17205::MAX17205<'static>,
//!     capsules::max17205::MAX17205::new(max17205_i2c_lower, max17205_i2c_upper,
//!                                       &mut capsules::max17205::BUFFER));
//! max17205_i2c.set_client(max17205);
//!
//! // For userspace.
//! let max17205_driver = static_init!(
//!     capsules::max17205::MAX17205Driver<'static>,
//!     capsules::max17205::MAX17205Driver::new(max17205));
//! max17205.set_client(max17205_driver);
//! ```

use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::hil::i2c;
use kernel::{AppId, Callback, Driver, ReturnCode};

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x80001;

pub static mut BUFFER: [u8; 8] = [0; 8];

// Addresses 0x000 - 0x0FF, 0x180 - 0x1FF can be written as blocks
// Addresses 0x100 - 0x17F must be written by word

// Addresses 0x000 - 0x0FF should use the i2c_lower device
// Addresses 0x100 - 0x1FF should use the i2c_upper device
enum Registers {
    Status = 0x000,
    RepCap = 0x005, // Reported capacity, LSB = 0.5 mAh
    //RepSOC = 0x006, // Reported capacity, LSB = %/256
    FullCapRep = 0x035, // Maximum capacity, LSB = 0.5 mAh
    //NPackCfg = 0x1B5, // Pack configuration
    NRomID = 0x1BC, //RomID - 64bit unique
    //NRSense = 0x1CF, // Sense resistor
    Batt = 0x0DA,    // Pack voltage, LSB = 1.25mV
    Current = 0x00A, // Instantaneous current, LSB = 156.25 uA
    Coulomb = 0x04D,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,

    /// Simple read states
    SetupReadCoulomb,
    ReadCoulomb,
    SetupReadStatus,
    ReadStatus,
    SetupReadSOC,
    ReadSOC,
    SetupReadCap,
    ReadCap,
    SetupReadVolt,
    ReadVolt,
    SetupReadCurrent,
    ReadCurrent,
    SetupReadRomID,
    ReadRomID,
}

pub trait MAX17205Client {
    fn status(&self, status: u16, error: ReturnCode);
    fn state_of_charge(&self, percent: u16, capacity: u16, full_capacity: u16, error: ReturnCode);
    fn voltage_current(&self, voltage: u16, current: u16, error: ReturnCode);
    fn coulomb(&self, coulomb: u16, error: ReturnCode);
    fn romid(&self, rid: u64, error: ReturnCode);
}

pub struct MAX17205<'a> {
    i2c_lower: &'a i2c::I2CDevice,
    i2c_upper: &'a i2c::I2CDevice,
    state: Cell<State>,
    soc: Cell<u16>,
    soc_mah: Cell<u16>,
    voltage: Cell<u16>,
    buffer: TakeCell<'static, [u8]>,
    client: Cell<Option<&'static MAX17205Client>>,
}

impl MAX17205<'a> {
    pub fn new(
        i2c_lower: &'a i2c::I2CDevice,
        i2c_upper: &'a i2c::I2CDevice,
        buffer: &'static mut [u8],
    ) -> MAX17205<'a> {
        MAX17205 {
            i2c_lower: i2c_lower,
            i2c_upper: i2c_upper,
            state: Cell::new(State::Idle),
            soc: Cell::new(0),
            soc_mah: Cell::new(0),
            voltage: Cell::new(0),
            buffer: TakeCell::new(buffer),
            client: Cell::new(None),
        }
    }

    pub fn set_client<C: MAX17205Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    fn setup_read_status(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c_lower.enable();

            buffer[0] = Registers::Status as u8;

            self.i2c_lower.write(buffer, 2);
            self.state.set(State::SetupReadStatus);

            ReturnCode::SUCCESS
        })
    }

    fn setup_read_soc(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c_lower.enable();

            // Get SOC mAh and percentage
            // Write reqcap address
            buffer[0] = Registers::RepCap as u8;
            self.i2c_lower.write(buffer, 1);
            self.state.set(State::SetupReadSOC);

            ReturnCode::SUCCESS
        })
    }

    fn setup_read_curvolt(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c_lower.enable();

            // Get current and voltage
            // Write Batt address
            buffer[0] = Registers::Batt as u8;
            self.i2c_lower.write(buffer, 1);
            self.state.set(State::SetupReadVolt);

            ReturnCode::SUCCESS
        })
    }

    fn setup_read_coulomb(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c_lower.enable();

            // Get raw coulomb count.
            // Write Coulomb address
            buffer[0] = Registers::Coulomb as u8;
            self.i2c_lower.write(buffer, 1);
            self.state.set(State::SetupReadCoulomb);

            ReturnCode::SUCCESS
        })
    }

    fn setup_read_romid(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c_upper.enable();

            buffer[0] = Registers::NRomID as u8;
            self.i2c_upper.write(buffer, 1);
            self.state.set(State::SetupReadRomID);

            ReturnCode::SUCCESS
        })
    }
}

impl i2c::I2CClient for MAX17205<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        match self.state.get() {
            State::SetupReadStatus => {
                // Read status
                self.i2c_lower.read(buffer, 2);
                self.state.set(State::ReadStatus);
            }
            State::ReadStatus => {
                let status = ((buffer[1] as u16) << 8) | (buffer[0] as u16);

                let error = if _error != i2c::Error::CommandComplete {
                    ReturnCode::ENOACK
                } else {
                    ReturnCode::SUCCESS
                };

                self.client.get().map(|client| client.status(status, error));

                self.buffer.replace(buffer);
                self.i2c_lower.disable();
                self.state.set(State::Idle);
            }
            State::SetupReadSOC => {
                // Write of SOC memory address complete, now issue read
                self.i2c_lower.read(buffer, 4);
                self.state.set(State::ReadSOC);
            }
            State::ReadSOC => {
                // Read of SOC memory address complete
                self.soc_mah
                    .set(((buffer[1] as u16) << 8) | (buffer[0] as u16));
                self.soc.set(((buffer[3] as u16) << 8) | (buffer[2] as u16));

                self.buffer.replace(buffer);

                // Now issue write of memory address of full capacity
                // Setup read capacity
                self.buffer.take().map(|selfbuf| {
                    // Get SOC mAh and percentage
                    // Write reqcap address
                    selfbuf[0] = ((Registers::FullCapRep as u8) & 0xFF) as u8;
                    self.i2c_lower.write(selfbuf, 1);

                    self.state.set(State::SetupReadCap);
                });
            }
            State::SetupReadCap => {
                // Now issue read
                self.i2c_lower.read(buffer, 2);
                self.state.set(State::ReadCap);
            }
            State::ReadCap => {
                let full_mah = ((buffer[1] as u16) << 8) | (buffer[0] as u16);

                let error = if _error != i2c::Error::CommandComplete {
                    ReturnCode::ENOACK
                } else {
                    ReturnCode::SUCCESS
                };

                self.client.get().map(|client| {
                    client.state_of_charge(self.soc.get(), self.soc_mah.get(), full_mah, error);
                });

                self.buffer.replace(buffer);
                self.i2c_lower.disable();
                self.state.set(State::Idle);
            }
            State::SetupReadCoulomb => {
                // Write of voltage memory address complete, now issue read
                self.i2c_lower.read(buffer, 2);
                self.state.set(State::ReadCoulomb);
            }
            State::ReadCoulomb => {
                // Read of voltage memory address complete
                let coulomb = ((buffer[1] as u16) << 8) | (buffer[0] as u16);

                let error = if _error != i2c::Error::CommandComplete {
                    ReturnCode::ENOACK
                } else {
                    ReturnCode::SUCCESS
                };

                self.client.get().map(|client| {
                    client.coulomb(coulomb, error);
                });

                self.buffer.replace(buffer);
                self.i2c_lower.disable();
                self.state.set(State::Idle);
            }
            State::SetupReadVolt => {
                // Write of voltage memory address complete, now issue read
                self.i2c_lower.read(buffer, 2);
                self.state.set(State::ReadVolt);
            }
            State::ReadVolt => {
                // Read of voltage memory address complete
                self.voltage
                    .set(((buffer[1] as u16) << 8) | (buffer[0] as u16));

                self.buffer.replace(buffer);

                // Now issue write of memory address of current
                // Setup read capacity
                self.buffer.take().map(|selfbuf| {
                    selfbuf[0] = ((Registers::Current as u8) & 0xFF) as u8;
                    self.i2c_lower.write(selfbuf, 1);

                    self.state.set(State::SetupReadCurrent);
                });
            }
            State::SetupReadCurrent => {
                // Now issue read
                self.i2c_lower.read(buffer, 2);
                self.state.set(State::ReadCurrent);
            }
            State::ReadCurrent => {
                let current = ((buffer[1] as u16) << 8) | (buffer[0] as u16);

                let error = if _error != i2c::Error::CommandComplete {
                    ReturnCode::ENOACK
                } else {
                    ReturnCode::SUCCESS
                };

                self.client
                    .get()
                    .map(|client| client.voltage_current(self.voltage.get(), current, error));

                self.buffer.replace(buffer);
                self.i2c_lower.disable();
                self.state.set(State::Idle);
            }
            State::SetupReadRomID => {
                self.i2c_upper.read(buffer, 8);
                self.state.set(State::ReadRomID);
            }
            State::ReadRomID => {
                // u64 from 8 bytes
                let rid = buffer
                    .iter()
                    .take(8)
                    .enumerate()
                    .fold(0u64, |rid, (i, b)| rid | ((*b as u64) << i * 8));
                self.buffer.replace(buffer);

                let error = if _error != i2c::Error::CommandComplete {
                    ReturnCode::ENOACK
                } else {
                    ReturnCode::SUCCESS
                };

                self.client.get().map(|client| client.romid(rid, error));

                self.i2c_upper.disable();
                self.state.set(State::Idle);
            }
            _ => {}
        }
    }
}

pub struct MAX17205Driver<'a> {
    max17205: &'a MAX17205<'a>,
    callback: Cell<Option<Callback>>,
}

impl MAX17205Driver<'a> {
    pub fn new(max: &'a MAX17205) -> MAX17205Driver<'a> {
        MAX17205Driver {
            max17205: max,
            callback: Cell::new(None),
        }
    }
}

impl MAX17205Client for MAX17205Driver<'a> {
    fn status(&self, status: u16, error: ReturnCode) {
        self.callback
            .get()
            .map(|mut cb| cb.schedule(From::from(error), status as usize, 0));
    }

    fn state_of_charge(&self, percent: u16, capacity: u16, full_capacity: u16, error: ReturnCode) {
        self.callback.get().map(|mut cb| {
            cb.schedule(
                From::from(error),
                percent as usize,
                (capacity as usize) << 16 | (full_capacity as usize),
            );
        });
    }

    fn voltage_current(&self, voltage: u16, current: u16, error: ReturnCode) {
        self.callback
            .get()
            .map(|mut cb| cb.schedule(From::from(error), voltage as usize, current as usize));
    }

    fn coulomb(&self, coulomb: u16, error: ReturnCode) {
        self.callback
            .get()
            .map(|mut cb| cb.schedule(From::from(error), coulomb as usize, 0));
    }

    fn romid(&self, rid: u64, error: ReturnCode) {
        self.callback.get().map(|mut cb| {
            cb.schedule(
                From::from(error),
                (rid & 0xffffffff) as usize,
                (rid >> 32) as usize,
            )
        });
    }
}

impl Driver for MAX17205Driver<'a> {
    /// Setup callback.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Setup a callback for when all events complete or data is ready.
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => {
                self.callback.set(callback);
                ReturnCode::SUCCESS
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Setup and read the MAX17205.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Read the current status of the MAX17205.
    /// - `2`: Read the current state of charge percent.
    /// - `3`: Read the current voltage and current draw.
    /// - `4`: Read the raw coulomb count.
    /// - `5`: Read the unique 64 bit RomID.
    fn command(&self, command_num: usize, _data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,

            // read status
            1 => self.max17205.setup_read_status(),

            // get soc
            2 => self.max17205.setup_read_soc(),

            // get voltage & current
            3 => self.max17205.setup_read_curvolt(),

            // get raw coulombs
            4 => self.max17205.setup_read_coulomb(),

            //
            5 => self.max17205.setup_read_romid(),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
