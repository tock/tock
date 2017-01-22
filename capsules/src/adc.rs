//! ADC Capsule
//!
//! Provides userspace applications with the ability to sample
//! ADC channels.

use core::cell::Cell;
use kernel::{AppId, Callback, Driver};
use kernel::hil::adc::{Client, AdcSingle};
use kernel::returncode::ReturnCode;

pub struct ADC<'a, A: AdcSingle + 'a> {
    adc: &'a A,
    channel: Cell<u8>,
    callback: Cell<Option<Callback>>,
}

impl<'a, A: AdcSingle + 'a> ADC<'a, A> {
    pub fn new(adc: &'a A) -> ADC<'a, A> {
        ADC {
            adc: adc,
            channel: Cell::new(0),
            callback: Cell::new(None),
        }
    }

    fn initialize(&self) -> ReturnCode {
        self.adc.initialize()
    }

    fn sample(&self, channel: u8) -> ReturnCode {
        self.channel.set(channel);
        self.adc.sample(channel)
    }
}

impl<'a, A: AdcSingle + 'a> Client for ADC<'a, A> {
    fn sample_done(&self, sample: u16) {
        self.callback.get().map(|mut cb| {
            cb.schedule(0, self.channel.get() as usize, sample as usize);
        });
    }
}

impl<'a, A: AdcSingle + 'a> Driver for ADC<'a, A> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // subscribe to ADC sample done
            0 => {
                self.callback.set(Some(callback));
                ReturnCode::SUCCESS
            }

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: AppId) -> ReturnCode {
        match command_num {
            // TODO: This should return the number of valid ADC channels.
            0 /* check if present */ => ReturnCode::SUCCESS,
            // Initialize ADC
            1 => self.initialize(),
            // Sample on channel
            2 => self.sample(data as u8),

            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
