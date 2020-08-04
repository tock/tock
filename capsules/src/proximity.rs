


use core::cell::Cell;
use kernel::hil;
use kernel::ReturnCode;
use kernel::{AppId, Callback, Driver, Grant};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Humidity as usize;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    subscribed: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ProximityCommand {
    Exists,
    ReadProximity,
    ReadProximityOnInterrupt,
    SetProximityGain,
    SetProximityInterruptThresholds,
}

pub struct ProximitySensor<'a> {
    driver: &'a dyn hil::sensors::ProximityDriver<'a>,
    apps: Grant<App>,
    busy: Cell<bool>,
}

impl<'a> ProximitySensor<'a> {
    pub fn new(
        driver : &'a dyn hil::sensors::ProximityDriver<'a>,
        grant: Grant<App>,
    ) -> ProximitySensor<'a> {
        ProximitySensor {
            driver: driver,
            apps: grant,
            busy: Cell::new(false),
        }
    }

    fn enqueue_command(&self, command: ProximityCommand, arg1: usize, arg2: usize, appid: AppId) -> ReturnCode {
        self.apps
            .enter(appid, |app, _| {
                if !self.busy.get() {
                    app.subscribed = true;
                    self.busy.set(true);
                    self.call_driver(command, arg1, arg2);
                } else {
                    ReturnCode::EBUSY
                }
            })
            .unwrap_or_else(|err| err.into())
    }

    fn call_driver(&self , command: ProximityCommand, arg1: usize, arg2: usize) -> ReturnCode{
        match command {
            ProximityCommand::ReadProximity => self.driver.read_proximity(),
            ProximityCommand::ReadProximityOnInterrupt => self.driver.read_proximity_on_interrupt(),
            ProximityCommand::SetProximityGain => self.driver.set_proximity_gain(arg1);
            ProximityCommand::SetProximityInterruptThresholds => self.driver.set_proximity_interrupt_thresholds(arg1, arg2);
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn configure_callback(&self, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        self.apps
            .enter(app_id, |app, _| {
                app.callback = callback;
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl hil::sensors::ProximityClient for ProximitySensor<'_> {
    fn callback(&self, tmp_val: usize) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {

                if app.subsribed {
                    self.busy.set(false);
                    app.subscribed = false;
                    app.callback.map(|mut cb| cb.schedule(tmp_val, 0, 0));
                }

            });
        }
    }
}

impl Driver for ProximitySensor<'_> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => self.configure_callback(callback, app_id),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, arg1: usize, arg2: usize, appid: AppId) -> ReturnCode {
        match command_num {
            // check whether the driver exist!!
            0 => ReturnCode::SUCCESS,

            // Instantaneous proximity measurement
            1 => self.enqueue_command(ProximityCommand::ReadProximity , arg1, arg2, appid);

            2 => self.enqueue_command(ProximityCommand::ReadProximityOnInterrupt , arg1, arg2, appid); 

            3 = > self.enqueue_command(ProximityCommand::SetProximityGain , arg1, arg2, appid);

            4 => self.enqueue_command(ProximityCommand::SetProximityInterruptThresholds , arg1, arg2, appid);

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

