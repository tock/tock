//! Provides userspace access to the analog comparators on a board.
//!
//! ## Instantiation
//!
//! ```rust let acifc = static_init!(
//! capsules::analog_comparator::AnalogComparator<'static, sam4l::acifc::Acifc>,
//! capsules::analog_comparator::AnalogComparator::new(&mut
//! sam4l::acifc::ACIFC)); ```
//!
//! ## Number of Analog Comparators
//! The number of analog comparators (ACs) available depends on the
//! microcontroller used.  For example, the Atmel SAM4L is a commonly used
//! microcontroller for Tock.  It comes in three different versions: a 48-pin, a
//! 64-pin and a 100-pin version.  On the 48-pin version, one AC is available.
//! On the 64-pin version, two ACs are available.  On the 100-pin version, four
//! ACs are available.  The Hail is an example of a board with the 64-pin
//! version of the SAM4L, and therefore supports two ACs.  These two ACs are
//! addressable by AC0 or AC1.  On the other hand, the Imix has a 100-pin
//! version of the SAM4L, and therefore supports four ACs.  These four ACs are
//! addressable by AC0, AC1, AC2 and AC3.
//!
//! ## Normal or Interrupt-based Comparison
//! For a normal comparison or an interrupt-based comparison, just one analog
//! comparator is necessary.
//!
//! ## Window Comparison
//! To do a window comparison, two ACs are necessary.  Therefore, the number
//! available windows on a microcontroller will be half the number of ACs.  For
//! instance, looking at the above "Number of Analog Comparators" explanation,
//! this means the Hail has one window and the Imix has two windows.
//!
//! For more information on how this capsule works, please take a look at the
//! readme: 00007_analog_comparator.md in doc/syscalls.

// Author: Danilo Verhaert <verhaert@cs.stanford.edu>
// Last modified 6/26/2018

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x00007;

use core::cell::Cell;
use kernel::hil;
use kernel::{AppId, Callback, Driver, ReturnCode};

pub struct AnalogComparator<'a, A: hil::analog_comparator::AnalogComparator + 'a> {
    ac: &'a A,
    callback: Cell<Option<Callback>>,
}

impl<'a, A: hil::analog_comparator::AnalogComparator> AnalogComparator<'a, A> {
    pub fn new(ac: &'a A) -> AnalogComparator<'a, A> {
        AnalogComparator {
            ac: ac,
            callback: Cell::new(None),
        }
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
    /// - `2`: Perform a window comparison.
    ///        Input x chooses the desired window Windowx (e.g. 0 for hail,
    ///        0 or 1 for imix)
    /// - `3`: Enable interrupt-based comparisons.
    ///        Input x chooses the desired comparator ACx (e.g. 0 or 1 for
    ///        hail, 0-3 for imix)
    /// - `4`: Disable interrupt-based comparisons.
    ///        Input x chooses the desired comparator ACx (e.g. 0 or 1 for
    ///        hail, 0-3 for imix)
    fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 => return ReturnCode::SUCCESS,

            1 => ReturnCode::SuccessWithValue {
                value: self.ac.comparison(data) as usize,
            },

            2 => ReturnCode::SuccessWithValue {
                value: self.ac.window_comparison(data) as usize,
            },

            3 => self.ac.enable_interrupts(data),

            4 => self.ac.disable_interrupts(data),

            _ => return ReturnCode::ENOSUPPORT,
        }
    }

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
    fn fired(&self) {
        // Callback to userland
        self.callback
            .get()
            .map_or_else(|| false, |mut cb| cb.schedule(0, 0, 0));
    }
}
