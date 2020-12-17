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

use core::cell::RefCell;
use kernel::hil;
use kernel::{AppId, Callback, CommandReturn, Driver, ErrorCode, ReturnCode};

pub struct AnalogComparator<'a, A: hil::analog_comparator::AnalogComparator<'a> + 'a> {
    // Analog Comparator driver
    analog_comparator: &'a A,
    channels: &'a [&'a <A as hil::analog_comparator::AnalogComparator<'a>>::Channel],

    // App state
    callback: RefCell<Callback>,
}

impl<'a, A: hil::analog_comparator::AnalogComparator<'a>> AnalogComparator<'a, A> {
    pub fn new(
        analog_comparator: &'a A,
        channels: &'a [&'a <A as hil::analog_comparator::AnalogComparator<'a>>::Channel],
    ) -> AnalogComparator<'a, A> {
        AnalogComparator {
            // Analog Comparator driver
            analog_comparator,
            channels,

            // App state
            callback: RefCell::new(Callback::default()),
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
    fn start_comparing(&self, channel: usize) -> ReturnCode {
        if channel >= self.channels.len() {
            return ReturnCode::EINVAL;
        }
        // Convert channel index
        let chan = self.channels[channel];
        let result = self.analog_comparator.start_comparing(chan);

        result
    }

    // Stop comparing on a channel
    fn stop_comparing(&self, channel: usize) -> ReturnCode {
        if channel >= self.channels.len() {
            return ReturnCode::EINVAL;
        }
        // Convert channel index
        let chan = self.channels[channel];
        let result = self.analog_comparator.stop_comparing(chan);

        result
    }
}

impl<'a, A: hil::analog_comparator::AnalogComparator<'a>> Driver for AnalogComparator<'a, A> {
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
    fn command(&self, command_num: usize, channel: usize, _: usize, _: AppId) -> CommandReturn {
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

    /// Provides a callback which can be used to signal the application
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Callback,
        _app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        match subscribe_num {
            // Subscribe to all interrupts
            0 => Ok(self.callback.replace(callback)),
            // Default
            _ => Err((callback, ErrorCode::NOSUPPORT)),
        }
    }
}

impl<'a, A: hil::analog_comparator::AnalogComparator<'a>> hil::analog_comparator::Client
    for AnalogComparator<'a, A>
{
    /// Callback to userland, signaling the application
    fn fired(&self, channel: usize) {
        self.callback.borrow_mut().schedule(channel, 0, 0);
    }
}
