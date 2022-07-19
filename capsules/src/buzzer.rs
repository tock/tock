//! Provides userspace access to a buzzer.
//!
//! ## Instantiation
//!
//! Instantiate the capsule for use as a syscall driver, using the corresponding service capsule.
//! For example, using the pwm buzzer:
//!
//! ``` rust
//!
//! let buzzer = static_init!(
//!     capsules::buzzer::Buzzer<'static>,
//!     capsules::buzzer::Buzzer::new(
//!         pwm_buzzer,
//!         board_kernel.create_grant(capsules::buzzer::DRIVER_NUM, &memory_allocation_capability)
//!     )
//! );
//!
//! ```

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{ErrorCode, ProcessId};

// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Buzzer as usize;

pub struct Buzzer<'a, B: hil::buzzer::Buzzer<'a>> {
    /// The service capsule buzzer.
    buzzer: &'a B,
    /// Per-app state.
    apps: Grant<(), UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    /// Which app is currently using the buzzer.
    active_app: OptionalCell<ProcessId>,
}

impl<'a, B: hil::buzzer::Buzzer<'a>> Buzzer<'a, B> {
    pub fn new(
        buzzer: &'a B,
        grant: Grant<(), UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Buzzer<'a, B> {
        Buzzer {
            buzzer: buzzer,
            apps: grant,
            active_app: OptionalCell::empty(),
        }
    }

    /// Checks whether an app is valid or not. The app is valid if
    /// there is no current active_app using the driver, or if the app corresponds
    /// to the current active_app. Otherwise, a different app is trying to
    /// use the driver while it is already in use, therefore it is not valid.
    pub fn is_valid_app(&self, appid: ProcessId) -> bool {
        self.active_app.map_or(
            true,
            |owning_app| {
                if owning_app == &appid {
                    true
                } else {
                    false
                }
            },
        )
    }
}

/// Provide an interface for userland.
impl<'a, B: hil::buzzer::Buzzer<'a>> SyscallDriver for Buzzer<'a, B> {
    // Setup callbacks.
    //
    // ### `subscribe_num`
    //
    // - `0`: Setup a buzz done callback.

    /// Command interface.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return Ok(()) if this driver is included on the platform.
    /// - `1`: Buzz the buzzer. `data1` is used for the frequency in hertz, and
    ///   `data2` is the duration in ms. Note the duration is capped at 5000
    ///   milliseconds.
    /// - `2`: Stop the buzzer.
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Check whether the driver exists.
            0 => CommandReturn::success(),

            // Play a sound.
            1 => {
                if !self.is_valid_app(appid) {
                    // A different app is trying to use the buzzer, so we return RESERVE.
                    CommandReturn::failure(ErrorCode::RESERVE)
                } else {
                    // If there is no active app or the same app is trying to use the buzzer,
                    // we set/replace the frequency and duration.
                    self.active_app.set(appid);
                    self.buzzer.buzz(data1, data2).into()
                }
            }

            // Stop the current sound.
            2 => {
                if !self.is_valid_app(appid) {
                    CommandReturn::failure(ErrorCode::RESERVE)
                } else if self.active_app.is_none() {
                    // If there is no active app, the buzzer isn't playing, so we return OFF.
                    CommandReturn::failure(ErrorCode::OFF)
                } else {
                    self.active_app.set(appid);
                    self.buzzer.stop().into()
                }
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl<'a, B: hil::buzzer::Buzzer<'a>> hil::buzzer::BuzzerClient for Buzzer<'a, B> {
    // The buzzer has finished playing its current sound.
    fn buzzer_done(&self, status: Result<(), ErrorCode>) {
        self.active_app.map(|c_app| {
            self.apps.enter(*c_app, |_app, upcalls| {
                upcalls
                    .schedule_upcall(0, (kernel::errorcode::into_statuscode(status), 0, 0))
                    .ok()
            })
        });
        // Remove the current app.
        self.active_app.clear();
    }
}
