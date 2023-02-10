//! This provides virtualized userspace access to a buzzer.
//!
//! Each app can have one outstanding buzz request, and buzz requests will queue
//! with each app getting exclusive access to the buzzer during its turn. Apps
//! can specify the frequency and duration of the square wave buzz, but the
//! duration is capped to prevent this from being annoying.
//!
//! Apps can subscribe to an optional callback if they care about getting
//! buzz done events.
//!
//! Usage
//! -----
//!
//! ```rust
//! # use kernel::static_init;
//!
//! let virtual_pwm_buzzer = static_init!(
//!     capsules::virtual_pwm::PwmPinUser<'static, nrf52::pwm::Pwm>,
//!     capsules::virtual_pwm::PwmPinUser::new(mux_pwm, nrf5x::pinmux::Pinmux::new(31))
//! );
//! virtual_pwm_buzzer.add_to_mux();
//!
//! let virtual_alarm_buzzer = static_init!(
//!     capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
//!     capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
//! );
//! virtual_alarm_buzzer.setup();
//!
//! let pwm_buzzer = static_init!(
//!     capsules::buzzer_pwm::PwmBuzzer<
//!         'static,
//!         capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>,
//!         capsules::virtual_pwm::PwmPinUser<'static, nrf52833::pwm::Pwm>,
//!     >,
//!     capsules::buzzer_pwm::PwmBuzzer::new(
//!         virtual_pwm_buzzer,
//!         virtual_alarm_buzzer,
//!         capsules::buzzer_pwm::DEFAULT_MAX_BUZZ_TIME_MS,
//!     )
//! );
//!
//! let buzzer_driver = static_init!(
//!     capsules::buzzer_driver::Buzzer<
//!         'static,
//!         capsules::buzzer_pwm::PwmBuzzer<
//!             'static,
//!             capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52833::rtc::Rtc>,
//!             capsules::virtual_pwm::PwmPinUser<'static, nrf52833::pwm::Pwm>,
//!         >,
//!     >,
//!     capsules::buzzer_driver::Buzzer::new(
//!         pwm_buzzer,
//!         capsules::buzzer_driver::DEFAULT_MAX_BUZZ_TIME_MS,
//!         board_kernel.create_grant(capsules::buzzer_driver::DRIVER_NUM, &memory_allocation_capability)
//!     )
//! );
//!
//! pwm_buzzer.set_client(buzzer_driver);
//!
//! virtual_alarm_buzzer.set_client(pwm_buzzer);
//! ```

use core::cmp;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use core_capsules::driver;
pub const DRIVER_NUM: usize = driver::NUM::Buzzer as usize;

/// Standard max buzz time.
pub const DEFAULT_MAX_BUZZ_TIME_MS: usize = 5000;

#[derive(Clone, Copy, PartialEq)]
pub enum BuzzerCommand {
    Buzz {
        frequency_hz: usize,
        duration_ms: usize,
    },
}

#[derive(Default)]
pub struct App {
    pending_command: Option<BuzzerCommand>, // What command to run when the buzzer is free.
}

pub struct Buzzer<'a, B: hil::buzzer::Buzzer<'a>> {
    /// The service capsule buzzer.
    buzzer: &'a B,
    /// Per-app state.
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    /// Which app is currently using the buzzer.
    active_app: OptionalCell<ProcessId>,
    /// Max buzz time.
    max_duration_ms: usize,
}

impl<'a, B: hil::buzzer::Buzzer<'a>> Buzzer<'a, B> {
    pub fn new(
        buzzer: &'a B,
        max_duration_ms: usize,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Buzzer<'a, B> {
        Buzzer {
            buzzer: buzzer,
            apps: grant,
            active_app: OptionalCell::empty(),
            max_duration_ms: max_duration_ms,
        }
    }

    // Check so see if we are doing something. If not, go ahead and do this
    // command. If so, this is queued and will be run when the pending
    // command completes.
    fn enqueue_command(
        &self,
        command: BuzzerCommand,
        processid: ProcessId,
    ) -> Result<(), ErrorCode> {
        if self.active_app.is_none() {
            // No app is currently using the buzzer, so we just use this app.
            self.active_app.set(processid);
            match command {
                BuzzerCommand::Buzz {
                    frequency_hz,
                    duration_ms,
                } => self.buzzer.buzz(frequency_hz, duration_ms),
            }
        } else {
            // There is an active app, so queue this request (if possible).
            self.apps
                .enter(processid, |app, _| {
                    // Some app is using the storage, we must wait.
                    if app.pending_command.is_some() {
                        // No more room in the queue, nowhere to store this
                        // request.
                        Err(ErrorCode::NOMEM)
                    } else {
                        // We can store this, so lets do it.
                        app.pending_command = Some(command);
                        Ok(())
                    }
                })
                .unwrap_or_else(|err| err.into())
        }
    }

    fn check_queue(&self) {
        for appiter in self.apps.iter() {
            let processid = appiter.processid();
            let started_command = appiter.enter(|app, _| {
                // If this app has a pending command let's use it.
                app.pending_command.take().map_or(false, |command| {
                    // Mark this driver as being in use.
                    self.active_app.set(processid);
                    // Actually make the buzz happen.
                    match command {
                        BuzzerCommand::Buzz {
                            frequency_hz,
                            duration_ms,
                        } => self.buzzer.buzz(frequency_hz, duration_ms) == Ok(()),
                    }
                })
            });
            if started_command {
                break;
            }
        }
    }

    /// For buzzing immediatelly
    /// Checks whether an app is valid or not. The app is valid if
    /// there is no current active_app using the driver, or if the app corresponds
    /// to the current active_app. Otherwise, a different app is trying to
    /// use the driver while it is already in use, therefore it is not valid.
    pub fn is_valid_app(&self, processid: ProcessId) -> bool {
        self.active_app.map_or(true, |owning_app| {
            if owning_app == &processid {
                true
            } else {
                false
            }
        })
    }
}

impl<'a, B: hil::buzzer::Buzzer<'a>> hil::buzzer::BuzzerClient for Buzzer<'a, B> {
    fn buzzer_done(&self, status: Result<(), ErrorCode>) {
        // Mark the active app as None and see if there is a callback.
        self.active_app.take().map(|processid| {
            let _ = self.apps.enter(processid, |_app, upcalls| {
                upcalls
                    .schedule_upcall(0, (kernel::errorcode::into_statuscode(status), 0, 0))
                    .ok();
            });
        });

        // Remove the current app.
        self.active_app.clear();

        // Check if there is anything else to do.
        self.check_queue();
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
    /// - `1`: Buzz the buzzer when available. `data1` is used for the frequency in hertz, and
    ///   `data2` is the duration in ms. Note the duration is capped at 5000
    ///   milliseconds.
    /// - `2`: Buzz the buzzer immediatelly. `data1` is used for the frequency in hertz, and
    ///   `data2` is the duration in ms. Note the duration is capped at 5000
    ///   milliseconds.
    /// - `3`: Stop the buzzer.
    fn command(
        &self,
        command_num: usize,
        data1: usize,
        data2: usize,
        processid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // Check whether the driver exists.
            0 => CommandReturn::success(),

            // Play a sound when available.
            1 => {
                let frequency_hz = data1;
                let duration_ms = cmp::min(data2, self.max_duration_ms);
                self.enqueue_command(
                    BuzzerCommand::Buzz {
                        frequency_hz,
                        duration_ms,
                    },
                    processid,
                )
                .into()
            }

            // Play a sound immediately.
            2 => {
                if !self.is_valid_app(processid) {
                    // A different app is trying to use the buzzer, so we return RESERVE.
                    CommandReturn::failure(ErrorCode::RESERVE)
                } else {
                    // If there is no active app or the same app is trying to use the buzzer,
                    // we set/replace the frequency and duration.
                    self.active_app.set(processid);
                    self.buzzer.buzz(data1, data2).into()
                }
            }

            // Stop the current sound.
            3 => {
                if !self.is_valid_app(processid) {
                    CommandReturn::failure(ErrorCode::RESERVE)
                } else if self.active_app.is_none() {
                    // If there is no active app, the buzzer isn't playing, so we return OFF.
                    CommandReturn::failure(ErrorCode::OFF)
                } else {
                    self.active_app.set(processid);
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
