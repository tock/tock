use core::cell::Cell;
use kernel::{AppId, Callback, Driver};
use kernel::common::math::{sqrtf32, get_errno};
use kernel::common::take_cell::TakeCell;
use kernel::hil::gpio::{Pin, InterruptMode, Client};
use kernel::hil::i2c;

const DEFAULT_SCALE: u8 = 0x0;

#[allow(dead_code)]
enum Registers {
    SensorStatus = 0x00,
    Out_X_MSB = 0x01,
    Out_X_LSB = 0x02,
    Out_Y_MSB = 0x03,
    Out_Y_LSB = 0x04,
    Out_Z_MSB = 0x05,
    Out_Z_LSB = 0x06,
    XYZ_Data_CFG = 0x0e, 
    Ctrl_Reg1 = 0x2a, 
}

pub struct FXOS8700CQ<'a> {
    i2c: &'a i2c::I2CDevice,
    scale: Cell<u8>,
    repeated_mode: Cell<bool>,
    callback: Cell<Option<Callback>>,
    buffer: TakeCell<&'static mut [u8]>,
}

impl<'a> FXOS8700CQ<'a> {
    pub fn new(i2c: &'a i2c::I2CDevice,
               buffer: &'static mut [u8])
               -> FXOS8700CQ<'a> {
        // setup and return struct
        FXOS8700CQ {
            i2c: i2c,
            scale: Cell::new(DEFAULT_SCALE),
            repeated_mode: Cell::new(false),
            callback: Cell::new(None),
            buffer: TakeCell::new(buffer),
        }
    }

    fn enable_sensor(&self, scale: u8) {
        // enable and configure FXOS8700CQ
        self.buffer.take().map(|buf| {
            // turn on i2c 
            self.i2c.enable();
            // configure accelerometer scale 
            buf[0] = Registers::XYZ_Data_CFG as u8; 
            buf[1] = scale as u8; 
            self.i2c.write(buf, 2);

            // TODO configure magnetometer

            // set to active mode 
            buffer[0] = Registers::Ctrl_Reg1 as u8; 
            self.i2c.read(buffer, 2);
            buffer[1] = buffer[1] | 0x01; 
			self.i2c.write(buf, 2);
        });
    }

    fn disable_sensor(&self, temperature: Option<f32>) {
        // TODO set to inactive 
    }

    fn enable_interrupts(&self) {
    	// ???
    }

    fn disable_interrupts(&self) {
    	// ???
    }
}

fn calculate_acceleration() -> f32 {
    0 
}

impl<'a> i2c::I2CClient for FXOS8700CQ<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        // TODO(alevy): handle protocol errors
        match self.protocol_state.get() {
            ProtocolState::Configure => {
                self.buffer.replace(buffer);
                self.enable_interrupts();
                self.i2c.disable();
                self.protocol_state.set(ProtocolState::Idle);
            }
            ProtocolState::Deconfigure(temperature) => {
                self.buffer.replace(buffer);
                self.disable_interrupts();
                self.i2c.disable();
                self.protocol_state.set(ProtocolState::Idle);
                temperature.map(|temp_val| {
                    self.callback
                        .get()
                        .map(|mut cb| cb.schedule(temp_val as usize, get_errno() as usize, 0));
                    self.callback.set(None);
                });
            }
            ProtocolState::SetRegSensorVoltage => {
                // Read sensor voltage register
                self.i2c.read(buffer, 2);
                self.protocol_state.set(ProtocolState::ReadingSensorVoltage);
            }
            ProtocolState::ReadingSensorVoltage => {
                let sensor_voltage = (((buffer[0] as u16) << 8) | buffer[1] as u16) as i16;

                // Select die temperature register
                buffer[0] = Registers::DieTemperature as u8;
                self.i2c.write(buffer, 1);

                self.protocol_state.set(ProtocolState::SetRegDieTemperature(sensor_voltage));
            }
            ProtocolState::SetRegDieTemperature(sensor_voltage) => {
                // Read die temperature register
                self.i2c.read(buffer, 2);
                self.protocol_state.set(ProtocolState::ReadingDieTemperature(sensor_voltage));
            }
            ProtocolState::ReadingDieTemperature(sensor_voltage) => {
                let die_temperature = (((buffer[0] as u16) << 8) | buffer[1] as u16) as i16;
                self.buffer.replace(buffer);

                let temp_val = calculate_temperature(sensor_voltage, die_temperature);

                // disable callback and sensing if in single-shot mode
                if self.repeated_mode.get() == false {
                    // disable temperature sensor. When disabling is finished, we will give the
                    // temperature to the callback.
                    self.disable_sensor(Some(temp_val));
                } else {
                    // send value to callback
                    self.callback
                        .get()
                        .map(|mut cb| cb.schedule(temp_val as usize, get_errno() as usize, 0));

                    self.i2c.disable();
                }
            }
            _ => {}
        }
    }
}

impl<'a> Client for FXOS8700CQ<'a> {
    // fn fired(&self, _: usize) {
    //     self.buffer.take().map(|buf| {
    //         // turn on i2c to send commands
    //         self.i2c.enable();

    //         // select sensor voltage register and read it
    //         buf[0] = Registers::SensorVoltage as u8;
    //         self.i2c.write(buf, 1);
    //         self.protocol_state.set(ProtocolState::SetRegSensorVoltage);
    //     });
    // }
}

impl<'a> Driver for FXOS8700CQ<'a> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            // single temperature reading with callback
            0 => {
                // single sample mode
                self.repeated_mode.set(false);

                // set callback function
                self.callback.set(Some(callback));

                // enable sensor
                self.enable_sensor(self.scale.get());

                0
            }

            // periodic acceleration reading subscription
            1 => {
                // periodic sampling mode
                self.repeated_mode.set(true);

                // set callback function
                self.callback.set(Some(callback));

                // enable sensor
                self.enable_sensor(self.scale.get());

                0
            }

            // default
            _ => -1,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: AppId) -> isize {
        match command_num {
            // set period for sensing
            0 => {
                // bounds check on the period
                if (data & 0xFFFFFFF8) != 0 {
                    return ERR_BAD_VALUE;
                }

                // set period value
                // TODO 
                // self.scale.set((data & 0x7) as u8);

                0
            }

            // unsubscribe callback
            1 => {
                // clear callback function
                self.callback.set(None);

                // disable sensor
                self.disable_sensor(None);

                0
            }

            // default
            _ => -1,
        }
    }
}
