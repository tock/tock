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

#[derive(Default)]
pub struct App {}

pub struct Buzzer<'a, B: hil::buzzer::Buzzer<'a>> {
    /// The service capsule buzzer.
    buzzer: &'a B,
    /// Per-app state.
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    /// Which app is currently using the buzzer.
    active_app: OptionalCell<ProcessId>,
}

impl<'a, B: hil::buzzer::Buzzer<'a>> Buzzer<'a, B> {
    pub fn new(
        buzzer: &'a B,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Buzzer<'a, B> {
        Buzzer {
            buzzer: buzzer,
            apps: grant,
            active_app: OptionalCell::empty(),
        }
    }

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
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 =>
            // Check whether the driver exists.
            {
                CommandReturn::success()
            }

            1 =>
            // Play a sound.
            {
                if !self.is_valid_app(appid) {
                    // A different app is trying to use the buzzer, so we return BUSY.
                    CommandReturn::failure(ErrorCode::BUSY)
                } else {
                    // If there is no active app or the same app is trying to use the buzzer,
                    // we set/replace the frequency and duration.
                    self.active_app.set(appid);
                    self.buzzer.buzz(data1, data2).into()
                }
            }

            2 =>
            // Stop the current sound.
            {
                if !self.is_valid_app(appid) {
                    CommandReturn::failure(ErrorCode::BUSY)
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
            self.apps
                .enter(*c_app, |_app, upcalls| {
                    if status == Ok(()) {
                        // There were no errors, so schedule an upcall.
                        upcalls.schedule_upcall(0, (0, 0, 0)).ok();
                    }
                    Ok(())
                })
                .unwrap_or_else(|err| err.into())
        });
        // Remove the current app.
        self.active_app.clear();
    }
}
