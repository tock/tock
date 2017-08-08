//! Driver for the Silicon Labs SI7021 temperature/humidity sensor.
//!
//! https://www.silabs.com/products/sensors/humidity-sensors/Pages/si7013-20-21.aspx
//!
//! > The Si7006/13/20/21/34 devices are Silicon Labs’ latest generation I2C
//! > relative humidity and temperature sensors. All members of this device
//! > family combine fully factory-calibrated humidity and temperature sensor
//! > elements with an analog to digital converter, signal processing and an I2C
//! > host interface. Patented use of industry-standard low-K polymer
//! > dielectrics provides excellent accuracy and long term stability, along
//! > with low drift and low hysteresis. The innovative CMOS design also offers
//! > the lowest power consumption in the industry for a relative humidity and
//! > temperature sensor. The Si7013/20/21/34 devices are designed for high-
//! > accuracy applications, while the Si7006 is targeted toward lower-accuracy
//! > applications that traditionally have used discrete RH/T sensors.
//!
//! Usage
//! -----
//!
//! ```rust
//! let si7021_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x40));
//! let si7021_virtual_alarm = static_init!(
//!     VirtualMuxAlarm<'static, sam4l::ast::Ast>,
//!     VirtualMuxAlarm::new(mux_alarm));
//! let si7021 = static_init!(
//!     capsules::si7021::SI7021<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
//!     capsules::si7021::SI7021::new(si7021_i2c,
//!         si7021_virtual_alarm,
//!         &mut capsules::si7021::BUFFER));
//! si7021_i2c.set_client(si7021);
//! si7021_virtual_alarm.set_client(si7021);
//! ```

use core::cell::Cell;
use kernel;
use kernel::ReturnCode;
use kernel::common::take_cell::TakeCell;
use kernel::hil::i2c;
use kernel::hil::time;
use kernel::hil::time::Frequency;

/// Syscall number
pub const DRIVER_NUM: usize = 0x70003;

// Buffer to use for I2C messages
pub static mut BUFFER: [u8; 14] = [0; 14];

#[allow(dead_code)]
enum Registers {
    MeasRelativeHumidityHoldMode = 0xe5,
    MeasRelativeHumidityNoHoldMode = 0xf5,
    MeasTemperatureHoldMode = 0xe3,
    MeasTemperatureNoHoldMode = 0xf3,
    ReadTemperaturePreviousRHMeasurement = 0xe0,
    Reset = 0xfe,
    WriteRHTUserRegister1 = 0xe6,
    ReadRHTUserRegister1 = 0xe7,
    WriteHeaterControlRegister = 0x51,
    ReadHeaterControlRegister = 0x11,
    ReadElectronicIdByteOneA = 0xfa,
    ReadElectronicIdByteOneB = 0x0f,
    ReadElectronicIdByteTwoA = 0xfc,
    ReadElectronicIdByteTwoB = 0xc9,
    ReadFirmwareVersionA = 0x84,
    ReadFirmwareVersionB = 0xb8,
}

/// States of the I2C protocol with the LPS331AP.
#[derive(Clone,Copy,PartialEq)]
enum State {
    Idle,
    WaitTemp,
    WaitRh,

    /// States to read the internal ID
    SelectElectronicId1,
    ReadElectronicId1,
    SelectElectronicId2,
    ReadElectronicId2,

    /// States to take the current measurement
    TakeTempMeasurementInit,
    TakeRhMeasurementInit,
    ReadRhMeasurement,
    ReadTempMeasurement,
    GotTempMeasurement,
    GotRhMeasurement,
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum OnDeck {
    Nothing,
    Temperature,
    Humidity,
}

pub struct SI7021<'a, A: time::Alarm + 'a> {
    i2c: &'a i2c::I2CDevice,
    alarm: &'a A,
    temp_callback: Cell<Option<&'static kernel::hil::sensors::TemperatureClient>>,
    humidity_callback: Cell<Option<&'static kernel::hil::sensors::HumidityClient>>,
    state: Cell<State>,
    on_deck: Cell<OnDeck>,
    buffer: TakeCell<'static, [u8]>,
}

impl<'a, A: time::Alarm + 'a> SI7021<'a, A> {
    pub fn new(i2c: &'a i2c::I2CDevice, alarm: &'a A, buffer: &'static mut [u8]) -> SI7021<'a, A> {
        // setup and return struct
        SI7021 {
            i2c: i2c,
            alarm: alarm,
            temp_callback: Cell::new(None),
            humidity_callback: Cell::new(None),
            state: Cell::new(State::Idle),
            on_deck: Cell::new(OnDeck::Nothing),
            buffer: TakeCell::new(buffer),
        }
    }

    pub fn read_id(&self) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            buffer[0] = Registers::ReadElectronicIdByteOneA as u8;
            buffer[1] = Registers::ReadElectronicIdByteOneB as u8;
            self.i2c.write(buffer, 2);
            self.state.set(State::SelectElectronicId1);
        });
    }

    fn init_measurement(&self, buffer: &'static mut [u8]) {
        let interval = (20 as u32) * <A::Frequency>::frequency() / 1000;

        let tics = self.alarm.now().wrapping_add(interval);
        self.alarm.set_alarm(tics);

        // Now wait for timer to expire
        self.buffer.replace(buffer);
        self.i2c.disable();
    }

    fn set_idle(&self, buffer: &'static mut [u8]) {
        self.buffer.replace(buffer);
        self.i2c.disable();
        self.state.set(State::Idle);
    }
}

impl<'a, A: time::Alarm + 'a> i2c::I2CClient for SI7021<'a, A> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        match self.state.get() {
            State::SelectElectronicId1 => {
                self.i2c.read(buffer, 8);
                self.state.set(State::ReadElectronicId1);
            }
            State::ReadElectronicId1 => {
                buffer[6] = buffer[0];
                buffer[7] = buffer[1];
                buffer[8] = buffer[2];
                buffer[9] = buffer[3];
                buffer[10] = buffer[4];
                buffer[11] = buffer[5];
                buffer[12] = buffer[6];
                buffer[13] = buffer[7];
                buffer[0] = Registers::ReadElectronicIdByteTwoA as u8;
                buffer[1] = Registers::ReadElectronicIdByteTwoB as u8;
                self.i2c.write(buffer, 2);
                self.state.set(State::SelectElectronicId2);
            }
            State::SelectElectronicId2 => {
                self.i2c.read(buffer, 6);
                self.state.set(State::ReadElectronicId2);
            }
            State::ReadElectronicId2 => {
                self.set_idle(buffer);
            }
            State::TakeTempMeasurementInit => {
                self.init_measurement(buffer);
                self.state.set(State::WaitTemp);
            }
            State::TakeRhMeasurementInit => {
                self.init_measurement(buffer);
                self.state.set(State::WaitRh);
            }
            State::ReadRhMeasurement => {
                self.i2c.read(buffer, 2);
                self.state.set(State::GotRhMeasurement);
            }
            State::ReadTempMeasurement => {
                self.i2c.read(buffer, 2);
                self.state.set(State::GotTempMeasurement);
            }
            State::GotTempMeasurement => {
                // Temperature in hundredths of degrees centigrade
                let temp_raw = (((buffer[0] as u32) << 8) | (buffer[1] as u32)) as u32;
                let temp = (((temp_raw * 17572) / 65536) - 4685) as i16;

                self.temp_callback
                    .get()
                    .map(|cb| cb.callback(temp as usize));

                match self.on_deck.get() {
                    OnDeck::Humidity => {
                        self.on_deck.set(OnDeck::Nothing);
                        buffer[0] = Registers::MeasRelativeHumidityNoHoldMode as u8;
                        self.i2c.write(buffer, 1);
                        self.state.set(State::TakeRhMeasurementInit);
                    }
                    _ => {
                        self.set_idle(buffer);
                    }
                }
            }
            State::GotRhMeasurement => {
                // Humidity in hundredths of percent
                let humidity_raw = (((buffer[0] as u32) << 8) | (buffer[1] as u32)) as u32;
                let humidity = (((humidity_raw * 125 * 100) / 65536) - 600) as u16;

                self.humidity_callback
                    .get()
                    .map(|cb| cb.callback(humidity as usize));
                match self.on_deck.get() {
                    OnDeck::Temperature => {
                        self.on_deck.set(OnDeck::Nothing);
                        buffer[0] = Registers::MeasTemperatureNoHoldMode as u8;
                        self.i2c.write(buffer, 1);
                        self.state.set(State::TakeTempMeasurementInit);
                    }
                    _ => {
                        self.set_idle(buffer);
                    }
                }
            }
            _ => {}
        }
    }
}


impl<'a, A: time::Alarm + 'a> kernel::hil::sensors::TemperatureDriver for SI7021<'a, A> {
    fn read_temperature(&self) -> kernel::ReturnCode {
        self.buffer
            .take()
            .map(|buffer| {
                // turn on i2c to send commands
                self.i2c.enable();

                buffer[0] = Registers::MeasTemperatureNoHoldMode as u8;
                self.i2c.write(buffer, 1);
                self.state.set(State::TakeTempMeasurementInit);
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|| if self.on_deck.get() != OnDeck::Nothing {
                ReturnCode::EBUSY
            } else {
                self.on_deck.set(OnDeck::Temperature);
                ReturnCode::SUCCESS
            })
    }

    fn set_client(&self, client: &'static kernel::hil::sensors::TemperatureClient) {
        self.temp_callback.set(Some(client));
    }
}

impl<'a, A: time::Alarm + 'a> kernel::hil::sensors::HumidityDriver for SI7021<'a, A> {
    fn read_humidity(&self) -> kernel::ReturnCode {
        self.buffer
            .take()
            .map(|buffer| {
                // turn on i2c to send commands
                self.i2c.enable();

                buffer[0] = Registers::MeasRelativeHumidityNoHoldMode as u8;
                self.i2c.write(buffer, 1);
                self.state.set(State::TakeRhMeasurementInit);
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|| if self.on_deck.get() != OnDeck::Nothing {
                ReturnCode::EBUSY
            } else {
                self.on_deck.set(OnDeck::Humidity);
                ReturnCode::SUCCESS
            })
    }

    fn set_client(&self, client: &'static kernel::hil::sensors::HumidityClient) {
        self.humidity_callback.set(Some(client));
    }
}

impl<'a, A: time::Alarm + 'a> time::Client for SI7021<'a, A> {
    fn fired(&self) {
        self.buffer.take().map(|buffer| {
            // turn on i2c to send commands
            self.i2c.enable();

            self.i2c.read(buffer, 2);
            match self.state.get() {
                State::WaitRh => self.state.set(State::ReadRhMeasurement),
                State::WaitTemp => self.state.set(State::ReadTempMeasurement),
                _ => (),
            }
        });
    }
}
