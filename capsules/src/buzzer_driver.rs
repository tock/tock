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
//! ```
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
//!
//! let buzzer = static_init!(
//!     capsules::buzzer_driver::Buzzer<
//!         'static,
//!         capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>>,
//!     capsules::buzzer_driver::Buzzer::new(
//!         virtual_pwm_buzzer,
//!         virtual_alarm_buzzer,
//!         capsules::buzzer_driver::DEFAULT_MAX_BUZZ_TIME_MS,
//!         kernel::Grant::create())
//! );
//! virtual_alarm_buzzer.set_client(buzzer);
//! ```

use core::cmp;

use kernel::common::cells::OptionalCell;
use kernel::hil;
use kernel::hil::time::Frequency;
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0x90000;

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
    callback: Option<Callback>, // Optional callback to signal when the buzzer event is over.
    pending_command: Option<BuzzerCommand>, // What command to run when the buzzer is free.
}

pub struct Buzzer<'a, A: hil::time::Alarm<'a>> {
    // The underlying PWM generator to make the buzzer buzz.
    pwm_pin: &'a dyn hil::pwm::PwmPin,
    // Alarm to stop the buzzer after some time.
    alarm: &'a A,
    // Per-app state.
    apps: Grant<App>,
    // Which app is currently using the buzzer.
    active_app: OptionalCell<AppId>,
    // Max buzz time.
    max_duration_ms: usize,
}

impl<A: hil::time::Alarm<'a>> Buzzer<'a, A> {
    pub fn new(
        pwm_pin: &'a dyn hil::pwm::PwmPin,
        alarm: &'a A,
        max_duration_ms: usize,
        grant: Grant<App>,
    ) -> Buzzer<'a, A> {
        Buzzer {
            pwm_pin: pwm_pin,
            alarm: alarm,
            apps: grant,
            active_app: OptionalCell::empty(),
            max_duration_ms: max_duration_ms,
        }
    }

    // Check so see if we are doing something. If not, go ahead and do this
    // command. If so, this is queued and will be run when the pending
    // command completes.
    fn enqueue_command(&self, command: BuzzerCommand, app_id: AppId) -> ReturnCode {
        if self.active_app.is_none() {
            // No app is currently using the buzzer, so we just use this app.
            self.active_app.set(app_id);
            self.buzz(command)
        } else {
            // There is an active app, so queue this request (if possible).
            self.apps
                .enter(app_id, |app, _| {
                    // Some app is using the storage, we must wait.
                    if app.pending_command.is_some() {
                        // No more room in the queue, nowhere to store this
                        // request.
                        ReturnCode::ENOMEM
                    } else {
                        // We can store this, so lets do it.
                        app.pending_command = Some(command);
                        ReturnCode::SUCCESS
                    }
                })
                .unwrap_or_else(|err| err.into())
        }
    }

    fn buzz(&self, command: BuzzerCommand) -> ReturnCode {
        match command {
            BuzzerCommand::Buzz {
                frequency_hz,
                duration_ms,
            } => {
                // Start the PWM output at the specified frequency with a 50%
                // duty cycle.
                let ret = self
                    .pwm_pin
                    .start(frequency_hz, self.pwm_pin.get_maximum_duty_cycle() / 2);
                if ret != ReturnCode::SUCCESS {
                    return ret;
                }

                // Now start a timer so we know when to stop the PWM.
                let interval = (duration_ms as u32) * <A::Frequency>::frequency() / 1000;
                let tics = self.alarm.now().wrapping_add(interval);
                self.alarm.set_alarm(tics);
                ReturnCode::SUCCESS
            }
        }
    }

    fn check_queue(&self) {
        for appiter in self.apps.iter() {
            let started_command = appiter.enter(|app, _| {
                // If this app has a pending command let's use it.
                app.pending_command.take().map_or(false, |command| {
                    // Mark this driver as being in use.
                    self.active_app.set(app.appid());
                    // Actually make the buzz happen.
                    self.buzz(command) == ReturnCode::SUCCESS
                })
            });
            if started_command {
                break;
            }
        }
    }
}

impl<A: hil::time::Alarm<'a>> hil::time::AlarmClient for Buzzer<'a, A> {
    fn fired(&self) {
        // All we have to do is stop the PWM and check if there are any pending
        // uses of the buzzer.
        self.pwm_pin.stop();
        // Mark the active app as None and see if there is a callback.
        self.active_app.take().map(|app_id| {
            let _ = self.apps.enter(app_id, |app, _| {
                app.callback.map(|mut cb| cb.schedule(0, 0, 0));
            });
        });

        // Check if there is anything else to do.
        self.check_queue();
    }
}

/// Provide an interface for userland.
impl<A: hil::time::Alarm<'a>> Driver for Buzzer<'a, A> {
    /// Setup callbacks.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Setup a buzz done callback.
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        self.apps
            .enter(app_id, |app, _| {
                match subscribe_num {
                    0 => app.callback = callback,
                    _ => return ReturnCode::ENOSUPPORT,
                }
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }

    /// Command interface.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return SUCCESS if this driver is included on the platform.
    /// - `1`: Buzz the buzzer. `arg1` is used for the frequency in hertz, and
    ///   `arg2` is the duration in ms. Note the duration is capped at 5000
    ///   milliseconds.
    fn command(&self, command_num: usize, arg1: usize, arg2: usize, appid: AppId) -> ReturnCode {
        match command_num {
            0 =>
            /* This driver exists. */
            {
                ReturnCode::SUCCESS
            }

            1 => {
                let frequency_hz = arg1;
                let duration_ms = cmp::min(arg2, self.max_duration_ms);
                self.enqueue_command(
                    BuzzerCommand::Buzz {
                        frequency_hz,
                        duration_ms,
                    },
                    appid,
                )
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
