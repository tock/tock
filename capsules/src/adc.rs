//! ADC Capsule
//!
//! Provides userspace applications with the ability to sample
//! ADC channels.

use core::cell::Cell;
use kernel::{AppId, Callback, Container, Driver, ReturnCode};
use kernel::hil::adc::{Client, AdcSingle, AdcContinuous, AdcContinuousFast, AdcContinuousVeryFast};

#[derive(Default)]
pub struct AppData {
    channel: Option<u8>,
    sd_callback: Option<Callback>,
    int_callback: Option<Callback>,
}

pub struct ADC<'a, A: AdcSingle + AdcContinuous + AdcContinuousFast + AdcContinuousVeryFast + 'a> {
    adc: &'a A,
    channel: Cell<Option<u8>>,
    app: Container<AppData>,
    mode: Cell<bool>,
}

impl<'a, A: AdcSingle + AdcContinuous + AdcContinuousFast + AdcContinuousVeryFast + 'a> ADC<'a, A> {
    pub fn new(adc: &'a A, container: Container<AppData>) -> ADC<'a, A> {
        ADC {
            adc: adc,
            channel: Cell::new(None),
            app: container,
            mode: Cell::new(false),
        }
    }

    fn initialize(&self) -> ReturnCode {
        self.adc.initialize()
    }

    fn sample(&self, channel: u8, appid: AppId) -> ReturnCode {
        self.mode.set(false);
        self.app
            .enter(appid, |app, _| {
                app.channel = Some(channel);

                if self.channel.get().is_none() {
                    self.channel.set(Some(channel));
                    self.adc.sample(channel)
                } else {
                    ReturnCode::SUCCESS
                }
            })
            .unwrap_or(ReturnCode::ENOMEM)
    }

    fn sample_continuous (&self, channel: u8, interval: u32, appid: AppId) -> ReturnCode {
        self.mode.set(true);
        self.app
            .enter(appid, |app, _| {
                app.channel = Some(channel);

                if self.channel.get().is_none() {
                    self.channel.set(Some(channel));
                    self.adc.sample_continuous(channel, interval)
                } else {
                    ReturnCode::SUCCESS
                }
            })
            .unwrap_or(ReturnCode::ENOMEM)
    }

    fn sample_continuous_fast (&self, channel: u8, interval: u32, appid: AppId) -> ReturnCode {
        self.mode.set(true);
        self.app
            .enter(appid, |app, _| {
                app.channel = Some(channel);

                if self.channel.get().is_none() {
                    self.channel.set(Some(channel));
                    self.adc.sample_continuous_fast(channel, interval)
                } else {
                    ReturnCode::SUCCESS
                }
            })
            .unwrap_or(ReturnCode::ENOMEM)
    }

    fn sample_continuous_very_fast (&self, channel: u8, interval: u32, appid: AppId) -> ReturnCode {
        self.mode.set(true);
        self.app
            .enter(appid, |app, _| {
                app.channel = Some(channel);

                if self.channel.get().is_none() {
                    self.channel.set(Some(channel));
                    self.adc.sample_continuous_very_fast(channel, interval)
                } else {
                    ReturnCode::SUCCESS
                }
            })
            .unwrap_or(ReturnCode::ENOMEM)
    }

    fn cancel_sampling (&self) -> ReturnCode {
        self.adc.cancel_sampling()
    }

    fn compute_interval (&self, interval: u32) -> u32{
        self.adc.compute_interval(interval)
    }

    fn compute_interval_fast (&self, interval: u32) -> u32{
        self.adc.compute_interval_fast(interval)
    }

    fn compute_interval_very_fast (&self, interval: u32) -> u32{
        self.adc.compute_interval_very_fast(interval)
    }
}

impl<'a, A: AdcSingle + AdcContinuous + AdcContinuousFast + AdcContinuousVeryFast + 'a> Client for ADC<'a, A> {
    fn sample_done(&self, sample: u16) {
        self.channel.get().map(|cur_channel| {
            if !self.mode.get() {
                self.channel.set(None);
            }
            self.app.each(|app| if app.channel == Some(cur_channel) {
                if !self.mode.get() {
                    app.channel = None;
                }
                app.sd_callback.map(|mut cb| cb.schedule(0, cur_channel as usize, sample as usize));
            } else if app.channel.is_some() {
                self.channel.set(app.channel);
            });
        });
        if !self.mode.get() {
            self.channel.get().map(|next_channel| { self.adc.sample(next_channel); });
        }
    }
}

impl<'a, A: AdcSingle + AdcContinuous + AdcContinuousFast + AdcContinuousVeryFast + 'a> Driver for ADC<'a, A> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // subscribe to ADC sample done
            0 => {
                self.app
                    .enter(callback.app_id(),
                           |app, _| { app.sd_callback = Some(callback); })
                    .unwrap_or(());
                ReturnCode::SUCCESS
            },
            1 => {
                self.app
                    .enter(callback.app_id(),
                           |app, _| { app.int_callback = Some(callback); })
                    .unwrap_or(());
                ReturnCode::SUCCESS
            }
                        

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, data: usize, appid: AppId) -> ReturnCode {
        match command_num {
            // TODO: This should return the number of valid ADC channels.
            0 /* check if present */ => ReturnCode::SUCCESS,
            // Initialize ADC
            1 => self.initialize(),
            // Sample on channel
            2 => {
                self.sample(data as u8, appid)
            },
            3 => {
                // Due to the 32-bit limit of the data parameter to the
                // `command()' system call, only the lower 24 bits of
                // interval are used, leaving 8 bits for CHANNEL.
                let channel = (data & 0xFF) as u8;
                let interval = (data >> 8) as u32;
                if self.compute_interval(0) <= interval {
                    self.sample_continuous(channel, interval, appid)
                } else if self.compute_interval_fast(0) <= interval {
                    self.sample_continuous_fast(channel, interval, appid)
                } else {
                    self.sample_continuous_very_fast(channel, interval, appid)
                }
            
            },
            4 => {
                self.cancel_sampling()
            },
            5 => {
                let interval = data as u32;
                let precise_interval;
                if self.compute_interval(0) <= interval {
                    precise_interval = self.compute_interval(interval) 
                } else if self.compute_interval_fast(0) <= interval {
                    precise_interval = self.compute_interval_fast(interval) 
                } else {
                    precise_interval = self.compute_interval_very_fast(interval) 
                }
                self.app.each(|app| { 
                    app.int_callback.map(|mut cb| cb.schedule(0, 0, precise_interval as usize));
                });
                ReturnCode::SUCCESS
            },

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
