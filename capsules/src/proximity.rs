//! Provides userspace with access to proximity sensors.
//!
//! Userspace Interface
//! -------------------
//!
//! ### `subscribe` System Call
//!
//! The `subscribe` system call supports the single `subscribe_number` zero,
//! which is used to provide a callback that will return back the result of
//! a proximity reading.
//! The `subscribe`call return codes indicate the following:
//!
//! * `Ok(())`: the callback been successfully been configured.
//! * `ENOSUPPORT`: Invalid allow_num.
//!
//!
//! ### `command` System Call
//!
//! The `command` system call support one argument `cmd` which is used to specify the specific
//! operation, currently the following cmd's are supported:
//!
//! * `0`: check whether the driver exist
//! * `1`: read proximity
//! * `2`: read proximity on interrupt
//!
//!
//! The possible return from the 'command' system call indicates the following:
//!
//! * `Ok(())`:    The operation has been successful.
//! * `BUSY`:      The driver is busy.
//! * `ENOSUPPORT`: Invalid `cmd`.
//!
//! Usage
//! -----
//!
//! You need a device that provides the `hil::sensors::ProximityDriver` trait.
//! Here is an example of how to set up a proximity sensor with the apds9960 IC
//!
//! ```rust
//! # use kernel::static_init;
//!
//!let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
//!
//!let proximity = static_init!(
//!   capsules::proximity::ProximitySensor<'static>,
//!   capsules::proximity::ProximitySensor::new(apds9960 , board_kernel.create_grant(&grant_cap)));
//!
//!kernel::hil::sensors::ProximityDriver::set_client(apds9960, proximity);
//! ```

use core::cell::Cell;

use kernel::grant::Grant;
use kernel::hil;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Proximity as usize;

#[derive(Default)]
pub struct App {
    subscribed: bool,
    enqueued_command_type: ProximityCommand,
    lower_proximity: u8,
    upper_proximity: u8,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ProximityCommand {
    ReadProximity = 1,
    ReadProximityOnInterrupt = 2,
    NoCommand = 3,
}

impl Default for ProximityCommand {
    fn default() -> Self {
        ProximityCommand::NoCommand
    }
}

#[derive(Default)]
pub struct Thresholds {
    lower: u8,
    upper: u8,
}

pub struct ProximitySensor<'a> {
    driver: &'a dyn hil::sensors::ProximityDriver<'a>,
    apps: Grant<App, 1>,
    command_running: Cell<ProximityCommand>,
}

impl<'a> ProximitySensor<'a> {
    pub fn new(
        driver: &'a dyn hil::sensors::ProximityDriver<'a>,
        grant: Grant<App, 1>,
    ) -> ProximitySensor<'a> {
        ProximitySensor {
            driver: driver,
            apps: grant,
            command_running: Cell::new(ProximityCommand::NoCommand),
        }
    }

    fn enqueue_command(
        &self,
        command: ProximityCommand,
        arg1: usize,
        arg2: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        // Enqueue command by saving command type, args, appid within app struct in grant region
        self.apps
            .enter(appid, |app, _| {
                // Return busy if same app attempts to enqueue second command before first one is "callbacked"
                if app.subscribed {
                    return CommandReturn::failure(ErrorCode::BUSY);
                }

                if command == ProximityCommand::ReadProximityOnInterrupt {
                    app.lower_proximity = arg1 as u8;
                    app.upper_proximity = arg2 as u8;
                }

                app.subscribed = true; // enqueue
                app.enqueued_command_type = command;

                // If driver is currently processing a ReadProximityOnInterrupt command then we allow the current ReadProximityOnInterrupt command
                // to interrupt it.  With new thresholds set, we can account for all apps waiting on ReadProximityOnInterrupt with different thresholds set
                // to all receive a callback when appropriate.
                // Doing so ensures that the app issuing the current command can have it serviced without having to wait for the previous command to fire.
                if (self.command_running.get() == ProximityCommand::ReadProximityOnInterrupt)
                    && (command == ProximityCommand::ReadProximityOnInterrupt)
                {
                    let mut t: Thresholds = self.find_thresholds();
                    if t.lower < app.lower_proximity {
                        t.lower = app.lower_proximity;
                    }
                    if t.upper > app.upper_proximity {
                        t.upper = app.upper_proximity;
                    }
                    let _ = self.driver.read_proximity_on_interrupt(t.lower, t.upper);
                    self.command_running
                        .set(ProximityCommand::ReadProximityOnInterrupt);
                    return CommandReturn::success();
                }

                // If driver is currently processing a ReadProximityOnInterrupt command and current command is a ReadProximity then
                // then command the driver to interrupt the former and replace with the latter.  The former will still be in the queue as the app region in the
                // grant will have the `subscribed` boolean field set
                if (self.command_running.get() == ProximityCommand::ReadProximityOnInterrupt)
                    && (command == ProximityCommand::ReadProximity)
                {
                    let _ = self.driver.read_proximity();
                    self.command_running.set(ProximityCommand::ReadProximity);
                    return CommandReturn::success();
                }

                if self.command_running.get() == ProximityCommand::NoCommand {
                    match app.enqueued_command_type {
                        ProximityCommand::ReadProximity => {
                            let _ = self.driver.read_proximity();
                        }
                        ProximityCommand::ReadProximityOnInterrupt => {
                            let mut t: Thresholds = self.find_thresholds();
                            if t.lower < app.lower_proximity {
                                t.lower = app.lower_proximity;
                            }
                            if t.upper > app.upper_proximity {
                                t.upper = app.upper_proximity;
                            }
                            let _ = self.driver.read_proximity_on_interrupt(t.lower, t.upper);
                            self.command_running
                                .set(ProximityCommand::ReadProximityOnInterrupt);
                        }
                        ProximityCommand::NoCommand => {}
                    }
                }

                CommandReturn::success()
            })
            .unwrap_or_else(|err| CommandReturn::failure(err.into()))
    }

    fn run_next_command(&self) -> Result<(), ErrorCode> {
        // Find thresholds before entering any grant regions
        let t: Thresholds = self.find_thresholds();
        // Find and run another command
        for cntr in self.apps.iter() {
            let break_flag = cntr.enter(|app, _| {
                if app.subscribed {
                    // run it
                    match app.enqueued_command_type {
                        ProximityCommand::ReadProximity => {
                            let _ = self.driver.read_proximity();
                            self.command_running.set(ProximityCommand::ReadProximity);
                        }
                        ProximityCommand::ReadProximityOnInterrupt => {
                            let _ = self.driver.read_proximity_on_interrupt(t.lower, t.upper);
                            self.command_running
                                .set(ProximityCommand::ReadProximityOnInterrupt);
                        }
                        ProximityCommand::NoCommand => {}
                    }
                    true
                } else {
                    false
                }
            });

            if break_flag {
                break;
            }
        }

        Ok(())
    }

    fn find_thresholds(&self) -> Thresholds {
        // Get the lowest upper prox and highest lower prox of all enqueued apps waiting on a readproximityoninterrupt command
        // With the IC thresholds set to these two values, we ensure to never miss an interrupt-causing proximity value for any of the
        // apps waiting on a proximity interrupt
        // Interrupts for thresholds t1,t2 where t1 < t2 are triggered when proximity > t2 or proximity < t1.
        let mut highest_lower_proximity: u8 = 0;
        let mut lowest_upper_proximity: u8 = 255;

        for cntr in self.apps.iter() {
            cntr.try_enter(|app, _| {
                if (app.lower_proximity > highest_lower_proximity)
                    && app.subscribed
                    && app.enqueued_command_type == ProximityCommand::ReadProximityOnInterrupt
                {
                    highest_lower_proximity = app.lower_proximity;
                }
                if (app.upper_proximity < lowest_upper_proximity)
                    && app.subscribed
                    && app.enqueued_command_type == ProximityCommand::ReadProximityOnInterrupt
                {
                    lowest_upper_proximity = app.upper_proximity;
                }
            });
        }

        // return values
        Thresholds {
            lower: highest_lower_proximity,
            upper: lowest_upper_proximity,
        }
    }
}

impl hil::sensors::ProximityClient for ProximitySensor<'_> {
    fn callback(&self, temp_val: u8) {
        // Here we callback the values only to the apps which are relevant for the callback
        // We also dequeue any command for a callback so as to remove it from the wait list and add other commands to continue

        // Schedule callbacks for appropriate apps (any apps waiting for a proximity command)
        // For apps waiting on an interrupt, the reading is checked against the upper and lower thresholds of the app's enqueued command
        // to notice if this reading will fulfill the app's command.
        // The reading is also delivered to any apps waiting on an immediate reading.
        for cntr in self.apps.iter() {
            cntr.enter(|app, upcalls| {
                if app.subscribed {
                    if app.enqueued_command_type == ProximityCommand::ReadProximityOnInterrupt {
                        // Case: ReadProximityOnInterrupt
                        // Only callback to those apps which we expect would want to know about this threshold reading.
                        if ((temp_val as u8) > app.upper_proximity)
                            || ((temp_val as u8) < app.lower_proximity)
                        {
                            upcalls.schedule_upcall(0, temp_val as usize, 0, 0).ok();
                            app.subscribed = false; // dequeue
                        }
                    } else {
                        // Case: ReadProximity
                        // Upcall to all apps waiting on read_proximity.
                        upcalls.schedule_upcall(0, temp_val as usize, 0, 0).ok();
                        app.subscribed = false; // dequeue
                    }
                }
            });
        }

        // No command is temporarily being run here as we have performed the callback for our last command
        self.command_running.set(ProximityCommand::NoCommand);

        // When we are done with callback (one command) then find another waiting command to run and run it
        let _ = self.run_next_command();
    }
}

impl SyscallDriver for ProximitySensor<'_> {
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        arg2: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            // check whether the driver exist!!
            0 => CommandReturn::success(),

            // Instantaneous proximity measurement
            1 => self.enqueue_command(ProximityCommand::ReadProximity, arg1, arg2, appid),

            // Upcall occurs only after interrupt is fired
            2 => self.enqueue_command(
                ProximityCommand::ReadProximityOnInterrupt,
                arg1,
                arg2,
                appid,
            ),

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(processid, |_, _| {})
    }
}
