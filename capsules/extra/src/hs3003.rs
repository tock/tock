use core::cell::Cell;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::sensors::{HumidityClient, HumidityDriver, TemperatureClient, TemperatureDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

const I2C_ADDRESS: u8 = 0x44;

pub struct Hs3003<'a, I: I2CDevice> {
    buffer: TakeCell<'static, [u8]>,
    i2c: &'a I,
    temperature_client: OptionalCell<&'a dyn TemperatureClient>,
    humidity_client: OptionalCell<&'a dyn HumidityClient>,
    state: Cell<State>,
    pending_temperature: Cell<bool>,
    pending_humidity: Cell<bool>,
}

impl<'a, I: I2CDevice> Hs3003<'a, I> {
    pub fn new(i2c: &'a I, buffer: &'static mut [u8]) -> Self {
        Hs3003 {
            buffer: TakeCell::new(buffer),
            i2c,
            temperature_client: OptionalCell::empty(),
            humidity_client: OptionalCell::empty(),
            state: Cell::new(State::Sleep(0, 0)),
            pending_temperature: Cell::new(false),
            pending_humidity: Cell::new(false),
        }
    }

    pub fn start_reading(&self) -> Result<(), ErrorCode> {
        self.buffer
            .take()
            .map(|buffer| {
                self.i2c.enable();
                match self.state.get() {
                    State::Sleep(_, _) => {
                        buffer[0] = I2C_ADDRESS << 1 | 0;

                        if let Err((_error, buffer)) = self.i2c.write(buffer, 1) {
                            self.buffer.replace(buffer);
                            self.i2c.disable();
                        } else {
                            self.state.set(State::InitiateReading);
                        }
                    }
                    _ => {}
                }
            })
            .ok_or(ErrorCode::BUSY)
    }
}

impl<'a, I: I2CDevice> TemperatureDriver<'a> for Hs3003<'a, I> {
    fn set_client(&self, client: &'a dyn TemperatureClient) {
        self.temperature_client.set(client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        self.pending_temperature.set(true);
        if !self.pending_humidity.get() {
            self.start_reading()
        } else {
            Ok(())
        }
    }
}

impl<'a, I: I2CDevice> HumidityDriver<'a> for Hs3003<'a, I> {
    fn set_client(&self, client: &'a dyn HumidityClient) {
        self.humidity_client.set(client);
    }

    fn read_humidity(&self) -> Result<(), ErrorCode> {
        self.pending_humidity.set(true);
        if !self.pending_temperature.get() {
            self.start_reading()
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum State {
    Sleep(i32, usize),
    InitiateReading,
    Read,
}

impl<'a, I: I2CDevice> I2CClient for Hs3003<'a, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if let Err(i2c_err) = status {
            self.state.set(State::Sleep(0, 0));
            self.buffer.replace(buffer);
            self.temperature_client
                .map(|client| client.callback(Err(i2c_err.into())));
            self.humidity_client.map(|client| client.callback(0));
            return;
        }

        match self.state.get() {
            State::InitiateReading => {
                buffer[0] = I2C_ADDRESS << 1 | 1;

                if let Err((i2c_err, buffer)) = self.i2c.write_read(buffer, 1, 4) {
                    self.state.set(State::Sleep(0, 0));
                    self.buffer.replace(buffer);
                    self.temperature_client
                        .map(|client| client.callback(Err(i2c_err.into())));
                    self.humidity_client.map(|client| client.callback(0));
                } else {
                    self.state.set(State::Read);
                }
            }
            State::Read => {
                let humidity_raw = (((buffer[0] & 0x3F) as u16) << 8) | buffer[1] as u16;
                let humidity = ((humidity_raw as f32 / ((1 << 14) - 1) as f32) * 100.0) as usize;

                let temperature_raw = buffer[2] as u16 | (buffer[3] as u16 >> 2);
                let temperature = ((temperature_raw as f32 / ((1 << 14) - 1) as f32) * 165.0 - 40.0) as i32;

                self.state.set(State::Sleep(temperature, humidity));
            }
            State::Sleep(temperature, humidity) => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                if self.pending_temperature.get() {
                    self.pending_temperature.set(false);
                    self.temperature_client
                        .map(|client| client.callback(Ok(temperature)));
                }
                if self.pending_humidity.get() {
                    self.pending_humidity.set(false);
                    self.humidity_client.map(|client| client.callback(humidity));
                }
            }
        }
    }
}