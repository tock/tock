use core::cell::Cell;
use common::math::{powi_f32, pow_f32, get_errno};
use hil::{Driver,Callback};
use hil::i2c::I2C;
use hil::gpio::{GPIOPin, InputMode, InterruptMode, Client};

// error codes for this driver
const ERR_BAD_VALUE: isize = -2;

const MAX_SAMPLING_RATE: u8 = 0x0;
const DEFAULT_SAMPLING_RATE: u8 = 0x02;

// temperature calculation constants
//  From TMP006 User's Guide section 5.1
//  S_0 should be determined from calibration and ranges from 5E-14 to 7E-14
//  We have selected 5E-14 experimentally
const S_0: f32 = 5E-14;
const A_1: f32 = 1.75E-3;
const A_2: f32 = -1.678E-5;
const T_REF: f32 = 298.15;
const B_0: f32 = -2.94E-5;
const B_1: f32 = -5.7E-7;
const B_2: f32 = 4.63E-9;
const C_2: f32 = 13.4;
const K_TO_C: f32 = -273.15;
const C_TO_K: f32 = 273.15;
const NV_TO_V: f32 = 1E9;
const T_DIE_CONVERT: f32 = 0.03125;
const V_OBJ_CONVERT: f32 = 156.25;

#[allow(dead_code)]
enum Registers {
    SensorVoltage = 0x00,
    DieTemperature = 0x01,
    Configuration = 0x02,
    ManufacturerID = 0xFE,
    DeviceID = 0xFF,
}

pub struct TMP006<'a, I: I2C + 'a, G: GPIOPin + 'a> {
    i2c: &'a I,
    i2c_address: Cell<u16>,
    interrupt_pin: &'a G,
    sampling_period: Cell<u8>,
    repeated_mode: Cell<bool>,
    callback: Cell<Option<Callback>>,
}

impl<'a, I: I2C, G: GPIOPin> TMP006<'a, I, G> {
    pub fn new(i2c: &'a I, i2c_address: u16, interrupt_pin: &'a G) -> TMP006<'a, I, G> {
        // setup and return struct
        TMP006{
            i2c: i2c,
            i2c_address: Cell::new(i2c_address),
            interrupt_pin: interrupt_pin,
            sampling_period: Cell::new(DEFAULT_SAMPLING_RATE),
            repeated_mode: Cell::new(false),
            callback: Cell::new(None),
        }
    }

    fn enable_sensor(&self, sampling_period: u8) {
        // turn on i2c to send commands
        self.i2c.enable();

        // enable and configure TMP006
        let mut buf: [u8; 3] = [0; 3];
        let config = 0x7100 | (((sampling_period & 0x7) as u16) << 9);
        buf[0] = Registers::Configuration as u8;
        buf[1] = ((config & 0xFF00) >> 8) as u8;
        buf[2] = (config & 0x00FF) as u8;
        self.i2c.write_sync(self.i2c_address.get(), &buf);

        // disable the i2c
        self.i2c.disable();
    }

    fn disable_sensor(&self) {
        // turn on i2c to send commands
        self.i2c.enable();

        // disable the TMP006
        let mut buf: [u8; 3] = [0; 3];
        let config = 0x0000;
        buf[0] = Registers::Configuration as u8;
        buf[1] = ((config & 0xFF00) >> 8) as u8;
        buf[2] = (config & 0x00FF) as u8;
        self.i2c.write_sync(self.i2c_address.get(), &buf);

        // disable the i2c
        self.i2c.disable();
    }

    fn enable_interrupts(&self) {
        // setup interrupts from the sensor
        self.interrupt_pin.enable_input(InputMode::PullUp);
        self.interrupt_pin.enable_interrupt(0, InterruptMode::FallingEdge);
    }

    fn disable_interrupts(&self) {
        // disable interrupts from the sensor
        self.interrupt_pin.disable_interrupt();
        self.interrupt_pin.disable();
    }

    #[allow(dead_code)]
    fn read_manufacturer_id(&self) -> u16 {
        // turn on i2c to send commands
        self.i2c.enable();

        let mut buf: [u8; 3] = [0; 3];

        // select manufacturer id register and read it
        buf[0] = Registers::ManufacturerID as u8;
        self.i2c.write_sync(self.i2c_address.get(), &buf[0..1]);
        self.i2c.read_sync(self.i2c_address.get(), &mut buf[0..2]);
        let manufacturer_id = (((buf[0] as u16) << 8) | buf[1] as u16) as u16;

        // disable i2c
        self.i2c.disable();

        // return device id
        manufacturer_id
    }

    #[allow(dead_code)]
    fn read_device_id(&self) -> u16 {
        // turn on i2c to send commands
        self.i2c.enable();

        let mut buf: [u8; 3] = [0; 3];

        // select device id register and read it
        buf[0] = Registers::DeviceID as u8;
        self.i2c.write_sync(self.i2c_address.get(), &buf[0..1]);
        self.i2c.read_sync(self.i2c_address.get(), &mut buf[0..2]);
        let device_id = (((buf[0] as u16) << 8) | buf[1] as u16) as u16;

        // disable i2c
        self.i2c.disable();

        // return device id
        device_id
    }

    fn read_temperature(&self) -> f32 {
        // turn on i2c to send commands
        self.i2c.enable();

        let mut buf: [u8; 3] = [0; 3];

        // select sensor voltage register and read it
        buf[0] = Registers::SensorVoltage as u8;
        self.i2c.write_sync(self.i2c_address.get(), &buf[0..1]);
        self.i2c.read_sync(self.i2c_address.get(), &mut buf[0..2]);
        let sensor_voltage = (((buf[0] as u16) << 8) | buf[1] as u16) as i16;

        // select die temperature register and read it
        buf[0] = Registers::DieTemperature as u8;
        self.i2c.write_sync(self.i2c_address.get(), &buf[0..1]);
        self.i2c.read_sync(self.i2c_address.get(), &mut buf[0..2]);
        let die_temperature = (((buf[0] as u16) << 8) | buf[1] as u16) as i16;

        // disable the i2c
        self.i2c.disable();

        // do calculation of actual temperature
        //  Calculations based on TMP006 User's Guide section 5.1
        let t_die = ((die_temperature >> 2) as f32)*T_DIE_CONVERT + C_TO_K;
        let t_adj = t_die-T_REF;
        let s = S_0 * (1.0 + A_1*t_adj + A_2*t_adj*t_adj);

        let v_obj = (sensor_voltage as f32)*V_OBJ_CONVERT/NV_TO_V;
        let v_os = B_0 + B_1*t_adj + B_2*t_adj*t_adj;

        let v_adj = v_obj-v_os;
        let f_v_obj = v_adj + C_2*v_adj*v_adj;

        let t_kelvin = pow_f32(powi_f32(t_die, 4) + (f_v_obj / s), 0.25);
        let t_celsius = t_kelvin + K_TO_C;

        // return data value
        t_celsius
    }
}

impl<'a, I: I2C, G: GPIOPin> Client for TMP006<'a, I, G> {
    fn fired(&self, _: usize) {
        // read value from temperature sensor
        let temp_val = self.read_temperature();

        // send value to callback
        if self.callback.get().is_some() {
            self.callback.get().unwrap().schedule(temp_val as usize, get_errno() as usize, 0);
        }

        // disable callback and sensing if in single-shot mode
        if self.repeated_mode.get() == false {
            // clear callback
            self.callback.set(None);

            // disable temperature sensor
            self.disable_sensor();
            self.disable_interrupts();
        }
    }
}

impl<'a, I: I2C, G: GPIOPin> Driver for TMP006<'a, I, G> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            // single temperature reading with callback
            0 => {
                // single sample mode
                self.repeated_mode.set(false);

                // set callback function
                self.callback.set(Some(callback));

                // enable sensor
                //  turn up the sampling rate so we get the sample faster
                self.enable_interrupts();
                self.enable_sensor(MAX_SAMPLING_RATE);

                0
            },

            // periodic temperature reading subscription
            1 => {
                // periodic sampling mode
                self.repeated_mode.set(true);

                // set callback function
                self.callback.set(Some(callback));

                // enable temperature sensor
                self.enable_interrupts();
                self.enable_sensor(self.sampling_period.get());

                0
            },

            // default
            _ => -1
        }
    }

    fn command(&self, command_num: usize, data: usize) -> isize {
        match command_num {
            // set period for sensing
            0 => {
                // bounds check on the period
                if (data & 0xFFFFFFF8) != 0 {
                    return ERR_BAD_VALUE;
                }

                // set period value
                self.sampling_period.set((data & 0x7) as u8);

                0
            },

            // unsubscribe callback
            1 => {
                // clear callback function
                self.callback.set(None);

                // disable temperature sensor
                self.disable_sensor();
                self.disable_interrupts();

                0
            },

            // default
            _ => -1
        }
    }
}

