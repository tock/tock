//! Provides a DAC interface for userspace.
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let dac = static_init!(
//!     capsules::dac::Dac<'static>,
//!     capsules::dac::Dac::new(&mut sam4l::dac::DAC));
//! ```

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Dac as usize;

use kernel::hil;
use kernel::{AppId, CommandResult, Driver, ErrorCode};

pub struct Dac<'a> {
    dac: &'a dyn hil::dac::DacChannel,
}

impl<'a> Dac<'a> {
    pub fn new(dac: &'a dyn hil::dac::DacChannel) -> Dac<'a> {
        Dac { dac: dac }
    }
}

impl Driver for Dac<'_> {
    /// Control the DAC.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Initialize and enable the DAC.
    /// - `2`: Set the output to `data1`, a scaled output value.
    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> CommandResult {
        match command_num {
            0 /* check if present */ => CommandResult::success(),

            // enable the dac
            1 => CommandResult::from(self.dac.initialize()),

            // set the dac output
            2 => CommandResult::from(self.dac.set_value(data)),

            _ => CommandResult::failure(ErrorCode::NOSUPPORT),
        }
    }
}
