//! Provides userspace access to the analog comparators on a board.
//!
//! Usage
//! -----
//!
//! ```
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
pub const DRIVER_NUM: usize = 0x00007;

use core::cell::Cell;
use kernel::hil;
use kernel::{AppId, Callback, Driver, ReturnCode};

pub struct AnalogComparator<'a, A: hil::analog_comparator::AnalogComparator + 'a> {
    // Analog Comparator driver
    analog_comparator: &'a A,
    channels: &'a [&'a <A as hil::analog_comparator::AnalogComparator>::Channel],

    // App state
    callback: Cell<Option<Callback>>,
}

impl<'a, A: hil::analog_comparator::AnalogComparator> AnalogComparator<'a, A> {
    pub fn new(
        analog_comparator: &'a A,
        channels: &'a [&'a <A as hil::analog_comparator::AnalogComparator>::Channel],
    ) -> AnalogComparator<'a, A> {
        AnalogComparator {
            // Analog Comparator driver
            analog_comparator: analog_comparator,
            channels: channels,

            // App state
            callback: Cell::new(None),
        }
    }

    // Do a single comparison on a channel
    fn comparison(&self, channel: usize) -> ReturnCode {
        if channel >= self.channels.len() {
            return ReturnCode::EINVAL;
        }
        // Convert channel index
        let chan = self.channels[channel];
        let result = self.analog_comparator.comparison(chan);

        return ReturnCode::SuccessWithValue {
            value: result as usize,
        };
    }

    // Start comparing on a channel
    fn start_comparing(&self, channel: usize) -> ReturnCode {
        if channel >= self.channels.len() {
            return ReturnCode::EINVAL;
        }
        // Convert channel index
        let chan = self.channels[channel];
        let result = self.analog_comparator.start_comparing(chan);

        return result;
    }

    // Stop comparing on a channel
    fn stop_comparing(&self, channel: usize) -> ReturnCode {
        if channel >= self.channels.len() {
            return ReturnCode::EINVAL;
        }
        // Convert channel index
        let chan = self.channels[channel];
        let result = self.analog_comparator.stop_comparing(chan);

        return result;
    }
}

impl<'a, A: hil::analog_comparator::AnalogComparator> Driver for AnalogComparator<'a, A> {
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
    fn command(&self, command_num: usize, channel: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SuccessWithValue {
                value: self.channels.len() as usize,
            },

            1 => self.comparison(channel),

            2 => self.start_comparing(channel),

            3 => self.stop_comparing(channel),

            _ => return ReturnCode::ENOSUPPORT,
        }
    }

    /// Provides a callback which can be used to signal the application
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // Subscribe to all interrupts
            0 => {
                self.callback.set(callback);
                ReturnCode::SUCCESS
            }
            // Default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a, A: hil::analog_comparator::AnalogComparator> hil::analog_comparator::Client
    for AnalogComparator<'a, A>
{
    /// Callback to userland, signaling the application
    fn fired(&self, channel: usize) {
        self.callback
            .get()
            .map_or_else(|| false, |mut cb| cb.schedule(channel, 0, 0));
    }
}
