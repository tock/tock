//! Driver for the PCA9544A I2C Selector.
//!
//! This chip allows for multiple I2C devices with the same addresses to
//! sit on the same I2C bus.
//!
//! <http://www.ti.com/product/PCA9544A>
//!
//! > The PCA9544A is a quad bidirectional translating switch controlled via the
//! > I2C bus. The SCL/SDA upstream pair fans out to four downstream pairs, or
//! > channels. One SCL/SDA pair can be selected at a time, and this is
//! > determined by the contents of the programmable control register. Four
//! > interrupt inputs (INT3–INT0), one for each of the downstream pairs, are
//! > provided. One interrupt output (INT) acts as an AND of the four interrupt
//! > inputs.
//!
//! Usage
//! -----
//!
//! ```rust
//! let pca9544a_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x70));
//! let pca9544a = static_init!(
//!     capsules::pca9544a::PCA9544A<'static>,
//!     capsules::pca9544a::PCA9544A::new(pca9544a_i2c, &mut capsules::pca9544a::BUFFER));
//! pca9544a_i2c.set_client(pca9544a);
//! ```

use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::hil::i2c;
use kernel::{AppId, Callback, Driver, ReturnCode};

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x80002;

pub static mut BUFFER: [u8; 5] = [0; 5];

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,

    /// Read the control register and return the specified data field.
    ReadControl(ControlField),

    Done,
}

#[derive(Clone, Copy, PartialEq)]
enum ControlField {
    InterruptMask,
    SelectedChannels,
}

pub struct PCA9544A<'a> {
    i2c: &'a i2c::I2CDevice,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    callback: Cell<Option<Callback>>,
}

impl<'a> PCA9544A<'a> {
    pub fn new(i2c: &'a i2c::I2CDevice, buffer: &'static mut [u8]) -> PCA9544A<'a> {
        PCA9544A {
            i2c: i2c,
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
            callback: Cell::new(None),
        }
    }

    /// Choose which channel(s) are active. Channels are encoded with a bitwise
    /// mask (0x01 means enable channel 0, 0x0F means enable all channels).
    /// Send 0 to disable all channels.
    fn select_channels(&self, channel_bitmask: u8) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c.enable();

            // Always clear the settings so we get to a known state
            buffer[0] = 0;

            // Iterate the bit array to send the correct channel enables
            let mut index = 1;
            for i in 0..4 {
                if channel_bitmask & (0x01 << i) != 0 {
                    // B2 B1 B0 are set starting at 0x04
                    buffer[index] = i + 4;
                    index += 1;
                }
            }

            self.i2c.write(buffer, index as u8);
            self.state.set(State::Done);

            ReturnCode::SUCCESS
        })
    }

    fn read_interrupts(&self) -> ReturnCode {
        self.read_control(ControlField::InterruptMask)
    }

    fn read_selected_channels(&self) -> ReturnCode {
        self.read_control(ControlField::SelectedChannels)
    }

    fn read_control(&self, field: ControlField) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c.enable();

            // Just issuing a read to the selector reads its control register.
            self.i2c.read(buffer, 1);
            self.state.set(State::ReadControl(field));

            ReturnCode::SUCCESS
        })
    }
}

impl<'a> i2c::I2CClient for PCA9544A<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        match self.state.get() {
            State::ReadControl(field) => {
                let ret = match field {
                    ControlField::InterruptMask => (buffer[0] >> 4) & 0x0F,
                    ControlField::SelectedChannels => buffer[0] & 0x07,
                };

                self.callback
                    .get()
                    .map(|mut cb| cb.schedule((field as usize) + 1, ret as usize, 0));

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::Done => {
                self.callback.get().map(|mut cb| cb.schedule(0, 0, 0));

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            _ => {}
        }
    }
}

impl<'a> Driver for PCA9544A<'a> {
    /// Setup callback for event done.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Callback is triggered when a channel is finished being selected
    ///   or when the current channel setup is returned.
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

    /// Control the I2C selector.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Choose which channels are active.
    /// - `2`: Disable all channels.
    /// - `3`: Read the list of fired interrupts.
    /// - `4`: Read which channels are selected.
    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            // Check if present.
            0 => ReturnCode::SUCCESS,

            // Select channels.
            1 => self.select_channels(data as u8),

            // Disable all channels.
            2 => self.select_channels(0),

            // Read the current interrupt fired mask.
            3 => self.read_interrupts(),

            // Read the current selected channels.
            4 => self.read_selected_channels(),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
