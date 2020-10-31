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
use kernel::{Driver, ProcessId, ReturnCode};

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
    fn command(&self, command_num: usize, data: usize, _: usize, _: ProcessId) -> ReturnCode {
        match command_num {
            0 /* check if present */ => ReturnCode::SUCCESS,

            // enable the dac
            1 => self.dac.initialize(),

            // set the dac output
            2 => self.dac.set_value(data),

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
