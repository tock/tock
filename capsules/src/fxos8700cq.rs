//! Driver for the FXOS8700CQ accelerometer.
//!
//! <http://www.nxp.com/assets/documents/data/en/data-sheets/FXOS8700CQ.pdf>
//!
//! The driver provides x, y, and z acceleration data to a callback function.
//! It implements the `hil::sensors::NineDof` trait.
//!
//! Usage
//! -----
//!
//! ```rust
//! let fxos8700_i2c = static_init!(I2CDevice, I2CDevice::new(i2c_bus, 0x1e));
//! let fxos8700 = static_init!(
//!     capsules::fxos8700cq::Fxos8700cq<'static>,
//!     capsules::fxos8700cq::Fxos8700cq::new(fxos8700_i2c,
//!                                           &sam4l::gpio::PA[9], // Interrupt pin
//!                                           &mut capsules::fxos8700cq::BUF));
//! fxos8700_i2c.set_client(fxos8700);
//! sam4l::gpio::PA[9].set_client(fxos8700);
//! ```

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil;
use kernel::hil::gpio;
use kernel::hil::i2c::{Error, I2CClient, I2CDevice};
use kernel::ReturnCode;

pub static mut BUF: [u8; 6] = [0; 6];

#[allow(dead_code)]
enum Registers {
    Status = 0x00,
    OutXMsb = 0x01,
    OutXLsb = 0x02,
    OutYMsb = 0x03,
    OutYLsb = 0x04,
    OutZMsb = 0x05,
    OutZLsb = 0x06,
    FSetup = 0x09,
    TrigCfg = 0x0a,
    Sysmod = 0x0b,
    IntSource = 0x0c,
    WhoAmI = 0x0d,
    XyzDataCfg = 0x0e,
    HpFilterCutoff = 0x0f,
    PlStatus = 0x10,
    PlCfg = 0x11,
    PlCount = 0x12,
    PlBfZcomp = 0x13,
    PlThsReg = 0x14,
    AFfmtCfg = 0x15,
    AFfmtSrc = 0x16,
    AFfmtThs = 0x17,
    AFfmtCount = 0x18,
    TransientCfg = 0x1d,
    TransientSrc = 0x1e,
    TransientThs = 0x1f,
    TransientCount = 0x20,
    PulseCfg = 0x21,
    PulseSrc = 0x22,
    PulseThsx = 0x23,
    PulseThsy = 0x24,
    PulseThsz = 0x25,
    PulseTmlt = 0x26,
    PulseLtcy = 0x27,
    PulseWind = 0x28,
    AslpCount = 0x29,
    CtrlReg1 = 0x2a,
    CtrlReg2 = 0x2b,
    CtrlReg3 = 0x2c,
    CtrlReg4 = 0x2d,
    CtrlReg5 = 0x2e,
    OffX = 0x2f,
    OffY = 0x30,
    OffZ = 0x31,
    MDrStatus = 0x32,
    MOutXMsb = 0x33,
    MOutXLsb = 0x34,
    MOutYMsb = 0x35,
    MOutYLsb = 0x36,
    MOutZMsb = 0x37,
    MOutZLsb = 0x38,
    CmpXMsb = 0x39,
    CmpXLsb = 0x3a,
    CmpYMsb = 0x3b,
    CmpYLsb = 0x3c,
    CmpZMsb = 0x3d,
    CmpZLsb = 0x3e,
    MOffXMsb = 0x3f,
    MOffXLsb = 0x40,
    MOffYMsb = 0x41,
    MOffYLsb = 0x42,
    MOffZMsb = 0x43,
    MOffZLsb = 0x44,
    MaxXMsb = 0x45,
    MaxXLsb = 0x46,
    MaxYMsb = 0x47,
    MaxYLsb = 0x48,
    MaxZMsb = 0x49,
    MaxZLsb = 0x4a,
    MinXMsb = 0x4b,
    MinXLsb = 0x4c,
    MinYMsb = 0x4d,
    MinYLsb = 0x4e,
    MinZMsb = 0x4f,
    MinZLsb = 0x50,
    Temp = 0x51,
    MThsCfg = 0x52,
    MThsSrc = 0x53,
    MThsXMsb = 0x54,
    MThsXLsb = 0x55,
    MThsYMsb = 0x56,
    MThsYLsb = 0x57,
    MThsZMsb = 0x58,
    MThsZLsb = 0x59,
    MThsCount = 0x5a,
    MCtrlReg1 = 0x5b,
    MCtrlReg2 = 0x5c,
    MCtrlReg3 = 0x5d,
    MIntSrc = 0x5e,
    AVecmCfg = 0x5f,
    AVecmThsMsb = 0x60,
    AVecmThsLsb = 0x61,
    AVecmCnt = 0x62,
    AVecmInitxMsb = 0x63,
    AVecmInitxLsb = 0x64,
    AVecmInityMsb = 0x65,
    AVecmInityLsb = 0x66,
    AVecmInitzMsb = 0x67,
    AVecmInitzLsb = 0x68,
    MVecmCfg = 0x69,
    MVecmThsMsb = 0x6a,
    MVecmThsLsb = 0x6b,
    MVecmCnt = 0x6c,
    MVecmInitxMsb = 0x6d,
    MVecmInitxLsb = 0x6e,
    MVecmInityMsb = 0x6f,
    MVecmInityLsb = 0x70,
    MVecmInitzMsb = 0x71,
    MVecmInitzLsb = 0x72,
    AFfmtThsXMsb = 0x73,
    AFfmtThsXLsb = 0x74,
    AFfmtThsYMsb = 0x75,
    AFfmtThsYLsb = 0x76,
    AFfmtThsZMsb = 0x77,
    AFfmtThsZLsb = 0x78,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    /// Sensor is in standby mode
    Disabled,

    /// Activate the accelerometer to take a reading
    ReadAccelSetup,

    /// Wait for the acceleration sample to be ready
    ReadAccelWait,

    /// Activate sensor to take readings
    ReadAccelWaiting,

    /// Reading accelerometer data
    ReadAccelReading,

    /// Deactivate sensor
    ReadAccelDeactivating(i16, i16, i16),

    /// Configuring reading the magnetometer
    ReadMagStart,

    /// Have the magnetometer values and sending them to application
    ReadMagValues,
}

pub struct Fxos8700cq<'a> {
    i2c: &'a I2CDevice,
    interrupt_pin1: &'a gpio::Pin,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    callback: OptionalCell<&'static hil::sensors::NineDofClient>,
}

impl Fxos8700cq<'a> {
    pub fn new(
        i2c: &'a I2CDevice,
        interrupt_pin1: &'a gpio::Pin,
        buffer: &'static mut [u8],
    ) -> Fxos8700cq<'a> {
        Fxos8700cq {
            i2c: i2c,
            interrupt_pin1: interrupt_pin1,
            state: Cell::new(State::Disabled),
            buffer: TakeCell::new(buffer),
            callback: OptionalCell::empty(),
        }
    }

    fn start_read_accel(&self) {
        // Need an interrupt pin
        self.interrupt_pin1.make_input();

        self.buffer.take().map(|buf| {
            self.i2c.enable();
            // Configure the data ready interrupt.
            buf[0] = Registers::CtrlReg4 as u8;
            buf[1] = 1; // CtrlReg4 data ready interrupt
            buf[2] = 1; // CtrlReg5 drdy on pin 1
            self.i2c.write(buf, 3);
            self.state.set(State::ReadAccelSetup);
        });
    }

    fn start_read_magnetometer(&self) {
        self.buffer.take().map(|buf| {
            self.i2c.enable();
            // Configure the magnetometer.
            buf[0] = Registers::MCtrlReg1 as u8;
            buf[1] = 0b00100001; // Enable magnetometer and one-shot read.
            self.i2c.write(buf, 2);
            self.state.set(State::ReadMagStart);
        });
    }
}

impl gpio::Client for Fxos8700cq<'a> {
    fn fired(&self, _: usize) {
        self.buffer.take().map(|buffer| {
            self.interrupt_pin1.disable_interrupt();

            // When we get this interrupt we can read the sample.
            self.i2c.enable();
            buffer[0] = Registers::OutXMsb as u8;
            self.i2c.write_read(buffer, 1, 6); // read 6 accel registers for xyz
            self.state.set(State::ReadAccelReading);
        });
    }
}

impl I2CClient for Fxos8700cq<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: Error) {
        match self.state.get() {
            State::ReadAccelSetup => {
                // Setup the interrupt so we know when the sample is ready
                self.interrupt_pin1
                    .enable_interrupt(0, gpio::InterruptMode::FallingEdge);

                // Enable the accelerometer.
                buffer[0] = Registers::CtrlReg1 as u8;
                buffer[1] = 1;
                self.i2c.write(buffer, 2);
                self.state.set(State::ReadAccelWait);
            }
            State::ReadAccelWait => {
                if self.interrupt_pin1.read() == false {
                    // Sample is already ready.
                    self.interrupt_pin1.disable_interrupt();
                    buffer[0] = Registers::OutXMsb as u8;
                    self.i2c.write_read(buffer, 1, 6); // read 6 accel registers for xyz
                    self.state.set(State::ReadAccelReading);
                } else {
                    // Wait for the interrupt to trigger
                    self.buffer.replace(buffer);
                    self.i2c.disable();
                    self.state.set(State::ReadAccelWaiting);
                }
            }
            State::ReadAccelReading => {
                let x = (((buffer[0] as i16) << 8) | buffer[1] as i16) >> 2;
                let y = (((buffer[2] as i16) << 8) | buffer[3] as i16) >> 2;
                let z = (((buffer[4] as i16) << 8) | buffer[5] as i16) >> 2;

                let x = ((x as isize) * 244) / 1000;
                let y = ((y as isize) * 244) / 1000;
                let z = ((z as isize) * 244) / 1000;

                // Now put the chip into standby mode.
                buffer[0] = Registers::CtrlReg1 as u8;
                buffer[1] = 0; // Set the active bit to 0.
                self.i2c.write(buffer, 2);
                self.state
                    .set(State::ReadAccelDeactivating(x as i16, y as i16, z as i16));
            }
            State::ReadAccelDeactivating(x, y, z) => {
                self.i2c.disable();
                self.state.set(State::Disabled);
                self.buffer.replace(buffer);
                self.callback.map(|cb| {
                    cb.callback(x as usize, y as usize, z as usize);
                });
            }
            State::ReadMagStart => {
                // One shot measurement taken, now read result.
                buffer[0] = Registers::MOutXMsb as u8;
                self.state.set(State::ReadMagValues);
                self.i2c.write_read(buffer, 1, 6);
            }
            State::ReadMagValues => {
                let x = (((buffer[0] as u16) << 8) | buffer[1] as u16) as i16;
                let y = (((buffer[2] as u16) << 8) | buffer[3] as u16) as i16;
                let z = (((buffer[4] as u16) << 8) | buffer[5] as u16) as i16;

                // Can immediately return values as the one-shot mode automatically
                // disables the fxo after taking the measurement.
                self.i2c.disable();
                self.state.set(State::Disabled);
                self.buffer.replace(buffer);

                self.callback
                    .map(|cb| cb.callback(x as usize, y as usize, z as usize));
            }
            _ => {}
        }
    }
}

impl hil::sensors::NineDof for Fxos8700cq<'a> {
    fn set_client(&self, client: &'static hil::sensors::NineDofClient) {
        self.callback.set(client);
    }

    fn read_accelerometer(&self) -> ReturnCode {
        self.start_read_accel();
        ReturnCode::SUCCESS
    }

    fn read_magnetometer(&self) -> ReturnCode {
        self.start_read_magnetometer();
        ReturnCode::SUCCESS
    }
}
