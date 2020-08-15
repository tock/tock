//! Proximity Sensor Interface to TockOS syscall interface

use core::cell::Cell;
use kernel::hil;
use kernel::ReturnCode;
use kernel::{AppId, Callback, Driver, Grant};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Proximity as usize;

#[derive(Default)]
pub struct App {
    callback: Option<Callback>,
    subscribed: bool,
    enqueued_command_type: ProximityCommand,
    lower_proximity: u8,
    upper_proximity: u8,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ProximityCommand {
    Exists = 0,
    ReadProximity = 1,
    ReadProximityOnInterrupt = 2,
}

impl Default for ProximityCommand {
    fn default() -> Self {ProximityCommand::Exists}
}

#[derive(Default)]
pub struct Thresholds {
    lower: u8,
    upper: u8,
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
        

        // Enqueue command by saving command type, args, appid within app struct in grant region
        let r: ReturnCode = self.apps.enter(appid, |app, _| {

            if !self.busy.get(){

                // Return busy if same app attempts to enqueue second command before first one is callbacked
                if app.subscribed {
                    self.busy.set(true);
                    return ReturnCode::EBUSY
                }

                if command == ProximityCommand::ReadProximityOnInterrupt{
                    app.lower_proximity = arg1 as u8;
                    app.upper_proximity = arg2 as u8;
                }

                app.subscribed = true; // enqueue
                app.enqueued_command_type = command;

                ReturnCode::SUCCESS

            } else {
                ReturnCode::EBUSY
            }

        }).unwrap_or_else(|err| err.into());

        if r == ReturnCode::EBUSY{ return ReturnCode::EBUSY }

        
        // Only run command if it is only one in queue otherwise we wait for callback() for last run command to trigger another command to run
        let mut num_commands: u8 = 0;

        for cntr in self.apps.iter(){
            cntr.enter(|app, _|{
                if app.subscribed {
                    num_commands += 1;
                }
            });
        }
        if num_commands == 1 {
            self.run_next_command();
        }

        ReturnCode::SUCCESS

    }

    fn  run_next_command(&self) -> ReturnCode {
        
        let mut break_flag: bool = false;

        // Find and run another command
        for cntr in self.apps.iter(){

            cntr.enter(|app, _|{
                if app.subscribed {
    
                    // run it
                    match app.enqueued_command_type {
                        ProximityCommand::ReadProximity => {
                            self.call_driver(app.enqueued_command_type , 0, 0);
                        }
                        ProximityCommand::ReadProximityOnInterrupt => {
                            let t: Thresholds = self.find_thresholds();
                            self.call_driver(app.enqueued_command_type, t.lower as usize , t.upper as usize );
                        }
                        _ => {}
                    }

                    break_flag = true;
                }

            });

            if break_flag { break; }
        }

        ReturnCode::SUCCESS
        
    }

    fn find_thresholds(&self) -> Thresholds {

        // Get the lowest upper prox and highest lower prox of all subscribed apps
        let mut highest_lower_proximity: u8 = 0;
        let mut lowest_upper_proximity: u8 = 255;

        for cntr in self.apps.iter(){
            cntr.enter(|app,_|{
                if (app.lower_proximity > highest_lower_proximity) && app.subscribed{
                    highest_lower_proximity = app.lower_proximity;
                }
                if (app.upper_proximity < lowest_upper_proximity) && app.subscribed{
                    lowest_upper_proximity = app.upper_proximity;
                }
            }); 
        }

        return Thresholds {
            lower: highest_lower_proximity,
            upper: lowest_upper_proximity,
        }
    }

    fn call_driver(&self , command: ProximityCommand, arg1: usize, arg2: usize) -> ReturnCode{
        match command {
            ProximityCommand::ReadProximity => self.driver.read_proximity(),
            ProximityCommand::ReadProximityOnInterrupt => self.driver.read_proximity_on_interrupt(arg1 as u8, arg2 as u8),
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
    fn callback(&self, temp_val: usize, command_type: usize) {
        
        // Here we callback the values only to the apps which are relevant for the callback
        // We also dequeue any command for a callback so as to remove it from the wait queue and add other commands to continue
        match command_type {
            command_type if command_type == ProximityCommand::ReadProximity as usize => {
                // Schedule callbacks for appropriate apps
                for cntr in self.apps.iter(){
                    cntr.enter(|app, _|{
                        if app.subscribed && (command_type == (app.enqueued_command_type as usize)){
                            app.callback.map(|mut cb| cb.schedule(temp_val, 0, 0));
                            app.subscribed = false; // dequeue
                        }
                    });
                }
            }

            command_type if command_type == ProximityCommand::ReadProximityOnInterrupt as usize => {
                // Schedule callbacks for appropriate apps
                for cntr in self.apps.iter(){
                    cntr.enter(|app, _|{
                        if app.subscribed && (command_type == (app.enqueued_command_type as usize)){
                            // Only callback to those apps which we expect would want to know about this threshold reading
                            if ((temp_val as u8) > app.upper_proximity) || ((temp_val as u8) < app.lower_proximity){
                                app.callback.map(|mut cb| cb.schedule(temp_val, 0, 0));
                                app.subscribed = false; // dequeue
                            }
                        }
                    });
                }

            }
            _ => {}
        }
        
        // When we are done with callback for one app then find another command to run and run it
        self.run_next_command();
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
            1 => self.enqueue_command(ProximityCommand::ReadProximity , arg1, arg2, appid),
            
            // Callback occurs only after interrupt is fired
            2 => self.enqueue_command(ProximityCommand::ReadProximityOnInterrupt , arg1, arg2, appid),

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}