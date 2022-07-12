//! Provides userspace access to a buzzer.

use kernel::hil;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::{ErrorCode, ProcessId};
use kernel::hil::buzzer::BuzzerCommand;

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Buzzer as usize;

#[derive(Default)]
pub struct App {
    pending_command: Option<BuzzerCommand>, // What command to run when the buzzer is free.
}

pub struct Buzzer<'a> {
    // The service capsule buzzer.
    buzzer: &'a dyn hil::buzzer::Buzzer<'a>,
    // Per-app state.
    apps: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    // Which app is currently using the buzzer.
    active_app: OptionalCell<ProcessId>,
}

impl<'a> Buzzer<'a> {
    pub fn new(
        buzzer: &'a dyn hil::buzzer::Buzzer<'a>,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    ) -> Buzzer<'a> {
        Buzzer {
            buzzer: buzzer,
            apps: grant,
            active_app: OptionalCell::empty(),
        }
    }

    // Check so see if we are doing something. If not, go ahead and do this
    // command. If so, this is queued and will be run when the pending
    // command completes.
    fn enqueue_command(&self, command: BuzzerCommand, app_id: ProcessId) -> Result<(), ErrorCode> {
        if self.active_app.is_none() {
            // No app is currently using the buzzer, so we just use this app.
            self.active_app.set(app_id);
            self.buzzer.buzz(command) //CHANGED! self.buzzer instead of just self
        } else {
            // There is an active app, so queue this request (if possible).
            self.apps
                .enter(app_id, |app, _| {
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

    /// Check to see if we have any more apps with commands waiting to be
    /// executed.
    fn check_queue(&self) {
        for appiter in self.apps.iter() {
            let appid = appiter.processid();
            let started_command = appiter.enter(|app, _| {
                /// If this app has a pending command let's use it.
                app.pending_command.take().map_or(false, |command| {
                    /// Mark this driver as being in use.
                    self.active_app.set(appid);
                    // Actually make the buzz happen.
                    self.buzzer.buzz(command) == Ok(())
                })
            });
            if started_command {
                break;
            }
        }
    }
}

/// Provide an interface for userland.
impl<'a> SyscallDriver for Buzzer<'a> {
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

            /// Play a sound.
            1 => {
                let frequency_hz = data1;
                let duration_ms = data2;
                self.enqueue_command(
                    BuzzerCommand::Buzz {
                        frequency_hz,
                        duration_ms,
                    },
                    appid,
                )
                .into()
            }

            /// Stop the current sound.
            2 => {
                self.buzzer.stop().into()
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}

impl hil::buzzer::BuzzerClient for Buzzer<'_> {
    /// The buzzer has finished playing its current sound.
    fn buzzer_done(&self, status: Result<(), ErrorCode>) {
        self.active_app.map(|c_app| {
            self.apps.enter(*c_app, |_app, upcalls| {
                if status == Ok(()) {
                    /// There were no errors, so schedule an upcall.
                    upcalls.schedule_upcall(0, (0, 0, 0)).ok();
                }
                Ok(())
            }).unwrap_or_else(|err| err.into())
        });
        /// Remove the current app.
        self.active_app.clear();
        /// Check queue for more commands that are waiting to be run.
        self.check_queue(); 
    }
}