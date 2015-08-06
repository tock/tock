use core::prelude::*;
use hil::{Driver,Callback};
use hil::i2c::I2C;
use hil::timer::*;

#[allow(dead_code)]
enum Registers {
    SensorVoltage = 0x00,
    LocalTemperature = 0x01,
    Configuration = 0x02,
    ManufacturerID = 0xFE,
    DeviceID = 0xFF
}

pub struct TMP006<I: I2C + 'static> {
    i2c: &'static mut I,
    timer: TimerMux,
    timer_request: Option<&'static mut TimerRequest>,
    callback: Option<Callback>
}

impl<I: I2C> TMP006<I> {
    pub fn new(i2c: &'static mut I, timer: TimerMux,
               timer_request: &'static mut TimerRequest)
            -> TMP006<I> {
        TMP006{i2c: i2c, timer: timer, timer_request: Some(timer_request), callback: None}
    }

    pub fn foo(&mut self) {
    }
}

impl<I: I2C> TimerCB for TMP006<I> {
    fn fired(&mut self, _: &'static mut TimerRequest, _: u32) {
        let mut buf: [u8; 3] = [0; 3];
        let mut config: u16;

        // Start by enabling the sensor
        config = 0x7 << 12;
        buf[0] = Registers::Configuration as u8;
        buf[1] = ((config & 0xFF00) >> 8) as u8;
        buf[2] = (config & 0x00FF) as u8;
        self.i2c.write_sync(0x40, &buf);


        // Wait for ready bit in control register
        loop {
            self.i2c.read_sync(0x40, &mut buf[0..2]);
            if buf[1] & 0x80 == 0x80 {
                break;
            }
        }

        // Now set the correct register pointer value so we can issue a read
        // to the sensor voltage register
        buf[0] = Registers::SensorVoltage as u8;
        self.i2c.write_sync(0x40, &buf[0..1]);

        // Now read the sensor reading
        self.i2c.read_sync(0x40, &mut buf[0..2]);
        //let sensor_voltage = (((buf[0] as u16) << 8) | buf[1] as u16) as i16;

        // Now move the register pointer to the die temp register
        buf[0] = Registers::LocalTemperature as u8;
        self.i2c.write_sync(0x40, &buf[0..1]);

        // Now read the 14bit die temp
        self.i2c.read_sync(0x40, &mut buf[0..2]);
        let die_temp = (((buf[0] as u16) << 8) | buf[1] as u16) as i16;

        // Shift to the right to make it 14 bits (this should be a signed shift)
        // The die temp is is in 1/32 degrees C.
        self.callback.as_mut().map(|cb| {
            cb.schedule((die_temp >> 2) as usize, 0, 0);
        });
    }
}

impl<I: I2C> Driver for TMP006<I> {
    fn subscribe(&'static mut self, subscribe_num: usize, mut callback: Callback) -> isize {
        match subscribe_num {
            0 /* read temperature  */ => {
                let mut buf: [u8; 3] = [0; 3];
                let mut config: u16;

                self.i2c.enable();
                self.callback = Some(callback);
                self.timer.repeat(32768, self.timer_request.take().unwrap());
                0
            },
            _ => -1
        }
    }

    fn command(&mut self, cmd_num: usize, _: usize) -> isize {
        match cmd_num {
            0 /* Enable sensor  */ => {
                let mut buf: [u8; 3] = [0; 3];
                let mut config: u16;

                self.i2c.enable();

                // Start by enabling the sensor
                config = 0x7 << 12;
                buf[0] = Registers::Configuration as u8;
                buf[1] = ((config & 0xFF00) >> 8) as u8;
                buf[2] = (config & 0x00FF) as u8;
                self.i2c.write_sync(0x40, &buf);

                0
            },
            _ => -1
        }
    }
}

