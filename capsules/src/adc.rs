//! ADC Capsule
//!
//! Provides userspace applications with the ability to sample
//! ADC channels.

use core::cell::Cell;
use kernel::{AppId, Callback, Container, Driver, ReturnCode};
use kernel::hil::adc::{Client, AdcSingle, AdcContinuous};

#[derive(Default)]
pub struct AppData {
    channel: Option<u8>,
    callback: Option<Callback>,
}

pub struct ADC<'a, A: AdcSingle + AdcContinuous + 'a> {
    adc: &'a A,
    channel: Cell<Option<u8>>,
    app: Container<AppData>,
    mode: Cell<bool>,
}

impl<'a, A: AdcSingle + AdcContinuous + 'a> ADC<'a, A> {
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

    fn sample_continuous (&self, channel: u8, frequency: u32, appid: AppId) -> ReturnCode {
        self.mode.set(true);
        self.app
            .enter(appid, |app, _| {
                app.channel = Some(channel);

                if self.channel.get().is_none() {
                    self.channel.set(Some(channel));
                    self.adc.sample_continuous(channel, frequency)
                } else {
                    ReturnCode::SUCCESS
                }
            })
            .unwrap_or(ReturnCode::ENOMEM)
    }
}

// The if statements are a hacky way to discriminate. Figure out a better way.
impl<'a, A: AdcSingle + AdcContinuous + 'a> Client for ADC<'a, A> {
    fn sample_done(&self, sample: u16) {
        self.channel.get().map(|cur_channel| {
            if !self.mode.get() {
                self.channel.set(None);
            }
            self.app.each(|app| if app.channel == Some(cur_channel) {
                if !self.mode.get() {
                    app.channel = None;
                }
                app.callback.map(|mut cb| cb.schedule(0, cur_channel as usize, sample as usize));
            } else if app.channel.is_some() {
                self.channel.set(app.channel);
            });
        });
        if !self.mode.get() {
            self.channel.get().map(|next_channel| { self.adc.sample(next_channel); });
        }
    }
}

impl<'a, A: AdcSingle + AdcContinuous + 'a> Driver for ADC<'a, A> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // subscribe to ADC sample done
            0 => {
                self.app
                    .enter(callback.app_id(),
                           |app, _| { app.callback = Some(callback); })
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
                // FREQUENCY are used, leaving 8 bits for CHANNEL.
                let channel = (data & 0xFF) as u8;
                let frequency = (data >> 8) as u32;
                self.sample_continuous(channel, frequency)
            },

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
