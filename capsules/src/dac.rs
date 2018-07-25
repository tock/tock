//! Provides a DAC interface for userspace.
//!
//! Usage
//! -----
//!
//! ```rust
//! let dac = static_init!(
//!     capsules::dac::Dac<'static>,
//!     capsules::dac::Dac::new(&mut sam4l::dac::DAC));
//! ```

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x00000006;

use kernel::hil;
use kernel::{AppId, Driver, ReturnCode};

pub struct Dac<'a> {
    dac: &'a hil::dac::DacChannel,
}

impl Dac<'a> {
    pub fn new(dac: &'a hil::dac::DacChannel) -> Dac<'a> {
        Dac { dac: dac }
    }
}

impl Driver for Dac<'a> {
    /// Control the DAC.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Initialize and enable the DAC.
    /// - `2`: Set the output to `data1`, a scaled output value.
    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 /* check if present */ => return ReturnCode::SUCCESS,

            // enable the dac
            1 => self.dac.initialize(),

            // set the dac output
            2 => self.dac.set_value(data),

            _ => return ReturnCode::ENOSUPPORT,
        }
    }
}
