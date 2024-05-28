// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Provides a DAC interface for userspace.
//!
//! Usage
//! -----
//!
//! ```rust,ignore
//! # use kernel::static_init;
//!
//! let dac = static_init!(
//!     capsules::dac::Dac<'static>,
//!     capsules::dac::Dac::new(&mut sam4l::dac::DAC));
//! ```

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Dac as usize;

use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

pub struct Dac<'a> {
    dac: &'a dyn hil::dac::DacChannel,
}

impl<'a> Dac<'a> {
    pub fn new(dac: &'a dyn hil::dac::DacChannel) -> Dac<'a> {
        Dac { dac: dac }
    }
}

impl SyscallDriver for Dac<'_> {
    /// Control the DAC.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver existence check.
    /// - `1`: Initialize and enable the DAC.
    /// - `2`: Set the output to `data1`, a scaled output value.
    fn command(&self, command_num: usize, data: usize, _: usize, _: ProcessId) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),

            // enable the dac. no-op as using the dac will enable it.
            1 => CommandReturn::success(),

            // set the dac output
            2 => CommandReturn::from(self.dac.set_value(data)),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, _processid: ProcessId) -> Result<(), kernel::process::Error> {
        Ok(())
    }
}
