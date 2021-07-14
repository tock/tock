use core::cell::Cell;

use kernel::hil::adc;
use kernel::hil::gpio;
use kernel::hil::sensors::{SoundPressure, SoundPressureClient};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::math;
use kernel::ErrorCode;

#[derive(Copy, Clone, PartialEq)]
enum State {
    Idle,
    ReadingSPL,
}

pub struct AdcMicrophone<'a, P: gpio::Pin> {
    adc: &'a dyn adc::AdcChannel,
    enable_pin: Option<&'a P>,
    spl_client: OptionalCell<&'a dyn SoundPressureClient>,
    spl_buffer: TakeCell<'a, [u16]>,
    spl_pos: Cell<usize>,
    state: Cell<State>,
}

impl<'a, P: gpio::Pin> AdcMicrophone<'a, P> {
    pub fn new(
        adc: &'a dyn adc::AdcChannel,
        enable_pin: Option<&'a P>,
        spl_buffer: &'a mut [u16],
    ) -> AdcMicrophone<'a, P> {
        enable_pin.map(|pin| pin.make_output());
        AdcMicrophone {
            adc,
            enable_pin,
            spl_client: OptionalCell::empty(),
            spl_buffer: TakeCell::new(spl_buffer),
            spl_pos: Cell::new(0),
            state: Cell::new(State::Idle),
        }
    }

    fn compute_spl(&self) -> u8 {
        let max = self.spl_buffer.map_or(0, |buffer| {
            let avg = (buffer.iter().fold(0usize, |a, v| a + *v as usize) / buffer.len()) as u16;
            let max = buffer
                .iter()
                .map(|v| if *v > avg { v - avg } else { 0 })
                .fold(0, |a, v| if a > v { a } else { v });
            let mut conv = (max as f32) / (((1 << 15) - 1) as f32) * 9 as f32;
            conv = 20f32 * math::log10(conv / 0.00002f32);
            conv as u8
        });
        max
    }
}

impl<'a, P: gpio::Pin> SoundPressure<'a> for AdcMicrophone<'a, P> {
    fn read_sound_pressure(&self) -> Result<(), ErrorCode> {
        if self.state.get() == State::Idle {
            // self.enable_pin.map (|pin| pin.set ());
            self.state.set(State::ReadingSPL);
            self.spl_pos.set(0);
            let _ = self.adc.sample();
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn set_client(&self, client: &'a dyn SoundPressureClient) {
        self.spl_client.set(client);
    }

    fn enable(&self) -> Result<(), ErrorCode> {
        self.enable_pin.map(|pin| pin.set());
        Ok(())
    }

    fn disable(&self) -> Result<(), ErrorCode> {
        self.enable_pin.map(|pin| pin.clear());
        Ok(())
    }
}

impl<'a, P: gpio::Pin> adc::Client for AdcMicrophone<'a, P> {
    fn sample_ready(&self, sample: u16) {
        if self.state.get() == State::ReadingSPL {
            if self.spl_buffer.map_or(false, |buffer| {
                if self.spl_pos.get() < buffer.len() {
                    buffer[self.spl_pos.get()] = sample;
                    self.spl_pos.set(self.spl_pos.get() + 1);
                }
                if self.spl_pos.get() < buffer.len() {
                    let _ = self.adc.sample();
                    false
                } else {
                    self.state.set(State::Idle);
                    true
                }
            }) {
                // self.enable_pin.map (|pin| pin.clear ());
                let spl = self.compute_spl();
                self.spl_client.map(|client| client.callback(Ok(()), spl));
            }
        }
    }
}
