//! SyscallDriver for the PCA9544A I2C Selector.
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
//! # use kernel::static_init;
//!
//! let pca9544a_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x70));
//! let pca9544a = static_init!(
//!     capsules::pca9544a::PCA9544A<'static>,
//!     capsules::pca9544a::PCA9544A::new(pca9544a_i2c, &mut capsules::pca9544a::BUFFER));
//! pca9544a_i2c.set_client(pca9544a);
//! ```

use core::cell::Cell;

use kernel::grant::Grant;
use kernel::hil::i2c;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Pca9544a as usize;

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

#[derive(Default)]
pub struct App {}

pub struct PCA9544A<'a> {
    i2c: &'a dyn i2c::I2CDevice,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    apps: Grant<App, 1>,
    owning_process: OptionalCell<ProcessId>,
}

impl<'a> PCA9544A<'a> {
    pub fn new(
        i2c: &'a dyn i2c::I2CDevice,
        buffer: &'static mut [u8],
        grant: Grant<App, 1>,
    ) -> Self {
        Self {
            i2c,
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
            apps: grant,
            owning_process: OptionalCell::empty(),
        }
    }

    /// Choose which channel(s) are active. Channels are encoded with a bitwise
    /// mask (0x01 means enable channel 0, 0x0F means enable all channels).
    /// Send 0 to disable all channels.
    fn select_channels(&self, channel_bitmask: u8) -> CommandReturn {
        self.buffer
            .take()
            .map_or(CommandReturn::failure(ErrorCode::NOMEM), |buffer| {
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

                // TODO verify errors
                let _ = self.i2c.write(buffer, index as u8);
                self.state.set(State::Done);

                CommandReturn::success()
            })
    }

    fn read_interrupts(&self) -> CommandReturn {
        self.read_control(ControlField::InterruptMask)
    }

    fn read_selected_channels(&self) -> CommandReturn {
        self.read_control(ControlField::SelectedChannels)
    }

    fn read_control(&self, field: ControlField) -> CommandReturn {
        self.buffer
            .take()
            .map_or(CommandReturn::failure(ErrorCode::NOMEM), |buffer| {
                self.i2c.enable();

                // Just issuing a read to the selector reads its control register.
                // TODO verify errors
                let _ = self.i2c.read(buffer, 1);
                self.state.set(State::ReadControl(field));

                CommandReturn::success()
            })
    }
}

impl i2c::I2CClient for PCA9544A<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], _status: Result<(), i2c::Error>) {
        match self.state.get() {
            State::ReadControl(field) => {
                let ret = match field {
                    ControlField::InterruptMask => (buffer[0] >> 4) & 0x0F,
                    ControlField::SelectedChannels => buffer[0] & 0x07,
                };

                self.owning_process.map(|pid| {
                    let _ = self.apps.enter(*pid, |_app, upcalls| {
                        upcalls
                            .schedule_upcall(0, (field as usize) + 1, ret as usize, 0)
                            .ok();
                    });
                });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            State::Done => {
                self.owning_process.map(|pid| {
                    let _ = self.apps.enter(*pid, |_app, upcalls| {
                        upcalls.schedule_upcall(0, 0, 0, 0).ok();
                    });
                });

                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Idle);
            }
            _ => {}
        }
    }
}

impl SyscallDriver for PCA9544A<'_> {
    // Setup callback for event done.
    //
    // ### `subscribe_num`
    //
    // - `0`: Upcall is triggered when a channel is finished being selected
    //   or when the current channel setup is returned.

    /// Control the I2C selector.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Choose which channels are active.
    /// - `2`: Disable all channels.
    /// - `3`: Read the list of fired interrupts.
    /// - `4`: Read which channels are selected.
    fn command(
        &self,
        command_num: usize,
        data: usize,
        _: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        if command_num == 0 {
            // Handle this first as it should be returned
            // unconditionally
            return CommandReturn::success();
        }
        // Check if this non-virtualized driver is already in use by
        // some (alive) process
        let match_or_empty_or_nonexistant = self.owning_process.map_or(true, |current_process| {
            self.apps
                .enter(*current_process, |_, _| current_process == &process_id)
                .unwrap_or(true)
        });
        if match_or_empty_or_nonexistant {
            self.owning_process.set(process_id);
        } else {
            return CommandReturn::failure(ErrorCode::NOMEM);
        }

        match command_num {
            // Check if present.
            0 => CommandReturn::success(),

            // Select channels.
            1 => self.select_channels(data as u8).into(),

            // Disable all channels.
            2 => self.select_channels(0).into(),

            // Read the current interrupt fired mask.
            3 => self.read_interrupts().into(),

            // Read the current selected channels.
            4 => self.read_selected_channels().into(),

            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
