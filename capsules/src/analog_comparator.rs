//! Provides userspace access to the analog comparators on a board.
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let ac_channels = static_init!(
//!     [&'static sam4l::acifc::AcChannel; 2],
//!     [
//!         &sam4l::acifc::CHANNEL_AC0,
//!         &sam4l::acifc::CHANNEL_AC1,
//!     ]
//! );
//! let analog_comparator = static_init!(
//!     capsules::analog_comparator::AnalogComparator<'static, sam4l::acifc::Acifc>,
//!     capsules::analog_comparator::AnalogComparator::new(&mut sam4l::acifc::ACIFC, ac_channels)
//! );
//! sam4l::acifc::ACIFC.set_client(analog_comparator);
//! ```
//!
//! ## Number of Analog Comparators
//! The number of analog comparators available depends on the microcontroller/board used.
//!
//! ## Normal or Interrupt-based Comparison
//! For a normal comparison or an interrupt-based comparison, just one analog
//! comparator is necessary.
//!
//! For more information on how this capsule works, please take a look at the
//! README: 00007_analog_comparator.md in doc/syscalls.

// Author: Danilo Verhaert <verhaert@cs.stanford.edu>
// Last modified August 9th, 2018

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::AnalogComparator as usize;

use kernel::grant::Grant;
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{ErrorCode, ProcessId};

pub struct AnalogComparator<'a, A: hil::analog_comparator::AnalogComparator<'a> + 'a> {
    // Analog Comparator driver
    analog_comparator: &'a A,
    channels: &'a [&'a <A as hil::analog_comparator::AnalogComparator<'a>>::Channel],

    grants: Grant<App, 1>,
    current_process: OptionalCell<ProcessId>,
}

#[derive(Default)]
pub struct App {}

impl<'a, A: hil::analog_comparator::AnalogComparator<'a>> AnalogComparator<'a, A> {
    pub fn new(
        analog_comparator: &'a A,
        channels: &'a [&'a <A as hil::analog_comparator::AnalogComparator<'a>>::Channel],
        grant: Grant<App, 1>,
    ) -> AnalogComparator<'a, A> {
        AnalogComparator {
            // Analog Comparator driver
            analog_comparator,
            channels,
            grants: grant,
            current_process: OptionalCell::empty(),
        }
    }

    // Do a single comparison on a channel
    fn comparison(&self, channel: usize) -> Result<bool, ErrorCode> {
        if channel >= self.channels.len() {
            return Err(ErrorCode::INVAL);
        }
        // Convert channel index
        let chan = self.channels[channel];
        let result = self.analog_comparator.comparison(chan);

        Ok(result)
    }

    // Start comparing on a channel
    fn start_comparing(&self, channel: usize) -> Result<(), ErrorCode> {
        if channel >= self.channels.len() {
            return Err(ErrorCode::INVAL);
        }
        // Convert channel index
        let chan = self.channels[channel];
        let result = self.analog_comparator.start_comparing(chan);

        result
    }

    // Stop comparing on a channel
    fn stop_comparing(&self, channel: usize) -> Result<(), ErrorCode> {
        if channel >= self.channels.len() {
            return Err(ErrorCode::INVAL);
        }
        // Convert channel index
        let chan = self.channels[channel];
        let result = self.analog_comparator.stop_comparing(chan);

        result
    }
}

impl<'a, A: hil::analog_comparator::AnalogComparator<'a>> SyscallDriver
    for AnalogComparator<'a, A>
{
    /// Control the analog comparator.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Driver check.
    /// - `1`: Perform a simple comparison.
    ///        Input x chooses the desired comparator ACx (e.g. 0 or 1 for
    ///        hail, 0-3 for imix)
    /// - `2`: Start interrupt-based comparisons.
    ///        Input x chooses the desired comparator ACx (e.g. 0 or 1 for
    ///        hail, 0-3 for imix)
    /// - `3`: Stop interrupt-based comparisons.
    ///        Input x chooses the desired comparator ACx (e.g. 0 or 1 for
    ///        hail, 0-3 for imix)
    fn command(
        &self,
        command_num: usize,
        channel: usize,
        _: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        if command_num == 0 {
            // Handle this first as it should be returned unconditionally.
            return CommandReturn::success_u32(self.channels.len() as u32);
        }

        // Check if this driver is free, or already dedicated to this process.
        let match_or_empty_or_nonexistant = self.current_process.map_or(true, |current_process| {
            self.grants
                .enter(*current_process, |_, _| current_process == &appid)
                .unwrap_or(true)
        });
        if match_or_empty_or_nonexistant {
            self.current_process.set(appid);
        } else {
            return CommandReturn::failure(ErrorCode::NOMEM);
        }

        match command_num {
            0 => CommandReturn::success_u32(self.channels.len() as u32),

            1 => match self.comparison(channel) {
                Ok(b) => CommandReturn::success_u32(b as u32),
                Err(e) => CommandReturn::failure(e),
            },

            2 => self.start_comparing(channel).into(),

            3 => self.stop_comparing(channel).into(),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.grants.enter(processid, |_, _| {})
    }
}

impl<'a, A: hil::analog_comparator::AnalogComparator<'a>> hil::analog_comparator::Client
    for AnalogComparator<'a, A>
{
    /// Upcall to userland, signaling the application
    fn fired(&self, channel: usize) {
        self.current_process.map(|appid| {
            let _ = self.grants.enter(*appid, |_app, upcalls| {
                upcalls.schedule_upcall(0, channel, 0, 0).ok();
            });
        });
    }
}
