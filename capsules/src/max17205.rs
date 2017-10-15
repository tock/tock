//! Driver for the Maxim MAX17205 fuel gauge.
//!
//! https://www.maximintegrated.com/en/products/power/battery-management/MAX17205.html
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
//! let max17205_i2c0 = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x36));
//! let max17205_i2c1 = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x0B));
//! let max17205 = static_init!(
//!     capsules::max17205::MAX17205<'static>,
//!     capsules::max17205::MAX17205::new(max17205_i2c0, max17205_i2c1,
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
use kernel::{AppSlice, AppId, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};
use kernel::hil::i2c;

pub static mut BUFFER: [u8; 8] = [0; 8];

// Addresses 0x000 - 0x0FF, 0x180 - 0x1FF can be written as blocks
// Addresses 0x100 - 0x17F must be written by word
enum Registers {
    Status = 0x000,
    RepCap = 0x005, // Reported capacity, LSB = 0.5 mAh
    RepSOC = 0x006, // Reported capacity, LSB = %/256
    FullCapRep = 0x035, // Maximum capacity, LSB = 0.5 mAh
    NPackCfg = 0x1B5, // Pack configuration
    NRomID = 0x1BC, //RomID - 64bit unique
    NRSense = 0x1CF, // Sense resistor
    Batt = 0x0DA, // Pack voltage, LSB = 1.25mV
    Current = 0x00A, // Instantaneous current, LSB = 156.25 uA
    Coulomb = 0x04D,
}

#[derive(Clone,Copy,PartialEq)]
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
    ReadRomID
}

pub trait MAX17205Client {
    fn status(&self, status: u16, error: ReturnCode);
    fn state_of_charge(&self, percent: u16, capacity: u16, full_capacity: u16, error: ReturnCode);
    fn voltage_current(&self, voltage: u16, current: u16, error: ReturnCode);
    fn coulomb(&self, coulomb: u16, error: ReturnCode);
    fn romid(&self, error: ReturnCode);
}


pub struct MAX17205<'a> {
    i2c0: &'a i2c::I2CDevice,
    i2c1: &'a i2c::I2CDevice,
    state: Cell<State>,
    soc: Cell<u16>,
    soc_mah: Cell<u16>,
    voltage: Cell<u16>,
    buffer: TakeCell<'static, [u8]>,
    rom_id_buffer: MapCell<AppSlice<Shared, u8>>,
    client: Cell<Option<&'static MAX17205Client>>,
}

impl<'a> MAX17205<'a> {
    pub fn new(i2c0: &'a i2c::I2CDevice, i2c1: &'a i2c::I2CDevice, buffer: &'static mut [u8]) -> MAX17205<'a> {
        MAX17205 {
            i2c0: i2c0,
            i2c1: i2c1,
            state: Cell::new(State::Idle),
            soc: Cell::new(0),
            soc_mah: Cell::new(0),
            voltage: Cell::new(0),
            buffer: TakeCell::new(buffer),
            rom_id_buffer: MapCell::empty(),
            client: Cell::new(None),
        }
    }

    pub fn set_client<C: MAX17205Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    fn setup_read_status(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c0.enable();

            buffer[0] = Registers::Status as u8;

            self.i2c0.write(buffer, 2);
            self.state.set(State::SetupReadStatus);

            ReturnCode::SUCCESS
        })
    }

    fn setup_read_soc(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c0.enable();

            // Get SOC mAh and percentage
            // Write reqcap address
            buffer[0] = Registers::RepCap as u8;
            self.i2c0.write(buffer, 1);
            self.state.set(State::SetupReadSOC);

            ReturnCode::SUCCESS
        })
    }

    fn setup_read_curvolt(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c0.enable();

            // Get current and voltage
            // Write Batt address
            buffer[0] = Registers::Batt as u8;
            self.i2c0.write(buffer, 1);
            self.state.set(State::SetupReadVolt);

            ReturnCode::SUCCESS
        })
    }

    fn setup_read_coulomb(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c0.enable();

            // Get raw coulomb count.
            // Write Coulomb address
            buffer[0] = Registers::Coulomb as u8;
            self.i2c0.write(buffer, 1);
            self.state.set(State::SetupReadCoulomb);

            ReturnCode::SUCCESS
        })
    }

    fn setup_read_romid(&self) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c1.enable();

            buffer[0] = Registers::NRomID as u8;
            self.i2c1.write(buffer, 1);
            self.state.set(State::SetupReadRomID);

            ReturnCode::SUCCESS
        })
    }
}

impl<'a> i2c::I2CClient for MAX17205<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {

        match self.state.get() {
            State::SetupReadStatus => {
                // Read status
                self.i2c0.read(buffer, 2);
                self.state.set(State::ReadStatus);
            }
            State::ReadStatus => {
                let status = ((buffer[1] as u16) << 8) | (buffer[0] as u16);

                let error = ReturnCode::SUCCESS;
                if _error != i2c::Error::CommandComplete {
                    error = ReturnCode::ENOACK;
                }

                self.client.get().map(|client| client.status(status, error));

                self.buffer.replace(buffer);
                self.i2c0.disable();
                self.state.set(State::Idle);
            }
            State::SetupReadSOC => {
                // Write of SOC memory address complete, now issue read
                self.i2c0.read(buffer, 4);
                self.state.set(State::ReadSOC);
            }
            State::ReadSOC => {
                // Read of SOC memory address complete
                self.soc_mah.set(((buffer[1] as u16) << 8) | (buffer[0] as u16));
                self.soc.set(((buffer[3] as u16) << 8) | (buffer[2] as u16));

                self.buffer.replace(buffer);

                // Now issue write of memory address of full capacity
                // Setup read capacity
                self.buffer.take().map(|selfbuf| {
                    // Get SOC mAh and percentage
                    // Write reqcap address
                    selfbuf[0] = ((Registers::FullCapRep as u8) & 0xFF) as u8;
                    self.i2c0.write(selfbuf, 1);

                    self.state.set(State::SetupReadCap);
                });
            }
            State::SetupReadCap => {
                // Now issue read
                self.i2c0.read(buffer, 2);
                self.state.set(State::ReadCap);
            }
            State::ReadCap => {
                let full_mah = ((buffer[1] as u16) << 8) | (buffer[0] as u16);

                let error = ReturnCode::SUCCESS;
                if _error != i2c::Error::CommandComplete {
                    error = ReturnCode::ENOACK;
                }

                self.client.get().map(|client| {
                    client.state_of_charge(self.soc.get(), self.soc_mah.get(), full_mah, error);
                });

                self.buffer.replace(buffer);
                self.i2c0.disable();
                self.state.set(State::Idle);
            }
            State::SetupReadCoulomb => {
                // Write of voltage memory address complete, now issue read
                self.i2c0.read(buffer, 2);
                self.state.set(State::ReadCoulomb);
            }
            State::ReadCoulomb => {
                // Read of voltage memory address complete
                let coulomb = ((buffer[1] as u16) << 8) | (buffer[0] as u16);

                let error = ReturnCode::SUCCESS;
                if _error != i2c::Error::CommandComplete {
                    error = ReturnCode::ENOACK;
                }

                self.client.get().map(|client| { client.coulomb(coulomb, error); });

                self.buffer.replace(buffer);
                self.i2c0.disable();
                self.state.set(State::Idle);
            }
            State::SetupReadVolt => {
                // Write of voltage memory address complete, now issue read
                self.i2c0.read(buffer, 2);
                self.state.set(State::ReadVolt);
            }
            State::ReadVolt => {
                // Read of voltage memory address complete
                self.voltage.set(((buffer[1] as u16) << 8) | (buffer[0] as u16));

                self.buffer.replace(buffer);

                // Now issue write of memory address of current
                // Setup read capacity
                self.buffer.take().map(|selfbuf| {
                    selfbuf[0] = ((Registers::Current as u8) & 0xFF) as u8;
                    self.i2c0.write(selfbuf, 1);

                    self.state.set(State::SetupReadCurrent);
                });
            }
            State::SetupReadCurrent => {
                // Now issue read
                self.i2c0.read(buffer, 2);
                self.state.set(State::ReadCurrent);
            }
            State::ReadCurrent => {
                let current = ((buffer[1] as u16) << 8) | (buffer[0] as u16);

                let error = ReturnCode::SUCCESS;
                if _error != i2c::Error::CommandComplete {
                    error = ReturnCode::ENOACK;
                }

                self.client
                    .get()
                    .map(|client| client.voltage_current(self.voltage.get(), current, error));

                self.buffer.replace(buffer);
                self.i2c0.disable();
                self.state.set(State::Idle);
            }
            State::SetupReadRomID => {
                self.i2c1.read(buffer, 8);
                self.state.set(State::ReadRomID);
            }
            State::ReadRomID => {
                let mut buf_len = 0;
                let exists = self.rom_id_buffer.map_or(false, |romid| {
                    buf_len = romid.map_or(0, |buffer| 8);
                    romid.is_some()
                });
                
                self.buffer.replace(buffer);

                let error = ReturnCode::SUCCESS;
                if _error != i2c::Error::CommandComplete {
                    error = ReturnCode::ENOACK;
                }

                if !exists {
                    error = ReturnCode::ENOMEM;
                }

                self.client.get().map(|client| client.romid(error));
                
                self.i2c1.disable();
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

impl<'a> MAX17205Driver<'a> {
    pub fn new(max: &'a MAX17205) -> MAX17205Driver<'a> {
        MAX17205Driver {
            max17205: max,
            callback: Cell::new(None),
        }
    }
}

impl<'a> MAX17205Client for MAX17205Driver<'a> {
    fn status(&self, status: u16, error: ReturnCode) {
        self.callback.get().map(|mut cb| cb.schedule(From::from(error), status as usize, 0));
    }

    fn state_of_charge(&self, percent: u16, capacity: u16, full_capacity: u16, error: ReturnCode) {
        self.callback.get().map(|mut cb| {
            cb.schedule(From::from(error),
                        percent as usize,
                        (capacity as usize) << 16 | (full_capacity as usize));
        });
    }

    fn voltage_current(&self, voltage: u16, current: u16, error: ReturnCode) {
        self.callback.get().map(|mut cb| cb.schedule(From::from(error), voltage as usize, current as usize));
    }

    fn coulomb(&self, coulomb: u16, error: ReturnCode) {
        self.callback.get().map(|mut cb| cb.schedule(From::from(error), coulomb as usize, 0));
    }

    fn romid(&self, error: ReturnCode) {
        self.callback.get().map(|mut cb| cb.schedule(From::from(error),0 , 0));
    }
}

impl<'a> Driver for MAX17205Driver<'a> {
    /// Setup callback.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Setup a callback for when all events complete or data is ready.
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            0 => {
                self.callback.set(Some(callback));
                ReturnCode::SUCCESS
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Allow buffer for the MAX17205
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Setup a buffer for the 64bit RomID
    fn allow(&self, _:AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            0 => {
                self.max17205.rom_id_buffer.map(|romid| {romid = Some(slice); });

                ReturnCode::SUCCESS
            }

            // default
            _=> ReturnCode::ENOSUPPORT,
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
