use crate::adc::AdcMode;
use crate::virtual_adc::Operation;
use kernel::common::cells::OptionalCell;
use kernel::hil;
use kernel::ReturnCode;
use kernel::{AppId, Callback, Driver, Grant};

pub struct App {
    callback: Option<Callback>,
    pending_command: bool,
    command: OptionalCell<Operation>,
    channel: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: None,
            pending_command: false,
            command: OptionalCell::empty(),
            channel: 0,
        }
    }
}

/// Multiplexed ADC Capsule for UserSpace
pub struct AdcSyscall<'a> {
    drivers: &'a [&'a dyn hil::adc::AdcChannel],
    apps: Grant<App>,
    current_app: OptionalCell<AppId>,
}

impl<'a> AdcSyscall<'a> {
    pub fn new(drivers: &'a [&'a dyn hil::adc::AdcChannel], grant: Grant<App>) -> AdcSyscall<'a> {
        AdcSyscall {
            drivers: drivers,
            apps: grant,
            current_app: OptionalCell::empty(),
        }
    }

    fn enqueue_command(&self, command: Operation, channel: usize, appid: AppId) -> ReturnCode {
        if channel < self.drivers.len() {
            self.apps
                .enter(appid, |app, _| {
                    if self.current_app.is_none() {
                        self.current_app.set(appid);
                        let value = self.call_driver(command, channel);
                        if value != ReturnCode::SUCCESS {
                            self.current_app.clear();
                        }
                        value
                    } else {
                        if app.pending_command == true {
                            ReturnCode::EBUSY
                        } else {
                            app.pending_command = true;
                            app.command.set(command);
                            app.channel = channel;
                            ReturnCode::SUCCESS
                        }
                    }
                })
                .unwrap_or_else(|err| err.into())
        } else {
            ReturnCode::ENODEVICE
        }
    }

    fn call_driver(&self, command: Operation, channel: usize) -> ReturnCode {
        match command {
            Operation::OneSample => self.drivers[channel].sample(),
        }
    }
}

impl Driver for AdcSyscall<'_> {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => self
                .apps
                .enter(app_id, |app, _| {
                    app.callback = callback;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, channel: usize, _: usize, appid: AppId) -> ReturnCode {
        match command_num {
            // This driver exists and return the number of channels
            0 => ReturnCode::SuccessWithValue {
                value: self.drivers.len() as usize,
            },

            // Single sample.
            1 => self.enqueue_command(Operation::OneSample, channel, appid),

            // Get resolution bits
            101 => {
                if channel < self.drivers.len() {
                    ReturnCode::SuccessWithValue {
                        value: self.drivers[channel].get_resolution_bits() as usize,
                    }
                } else {
                    ReturnCode::ENODEVICE
                }
            }

            // Get voltage reference mV
            102 => {
                if channel < self.drivers.len() {
                    if let Some(voltage) = self.drivers[channel].get_voltage_reference_mv() {
                        ReturnCode::SuccessWithValue {
                            value: voltage as usize,
                        }
                    } else {
                        ReturnCode::ENOSUPPORT
                    }
                } else {
                    ReturnCode::ENODEVICE
                }
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a> hil::adc::Client for AdcSyscall<'a> {
    fn sample_ready(&self, sample: u16) {
        self.current_app.take().map(|appid| {
            let _ = self.apps.enter(appid, |app, _| {
                app.pending_command = false;
                app.callback.map(|mut cb| {
                    cb.schedule(AdcMode::SingleSample as usize, app.channel, sample as usize);
                });
            });
        });
    }
}
