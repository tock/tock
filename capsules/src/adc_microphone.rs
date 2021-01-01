use core::cell::Cell;
use core::f32;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::adc;
use kernel::hil::gpio;
use kernel::hil::sensors::{SoundPressure, SoundPressureClient};
use kernel::ReturnCode;

// f32 log10 function adapted from [micromath](https://github.com/NeoBirth/micromath)
const EXPONENT_MASK: u32 = 0b01111111_10000000_00000000_00000000;
const EXPONENT_BIAS: u32 = 127;

fn abs(n: f32) -> f32 {
    f32::from_bits(n.to_bits() & 0x7FFF_FFFF)
}

fn extract_exponent_bits(x: f32) -> u32 {
    (x.to_bits() & EXPONENT_MASK).overflowing_shr(23).0
}

fn extract_exponent_value(x: f32) -> i32 {
    (extract_exponent_bits(x) as i32) - EXPONENT_BIAS as i32
}

fn ln_1to2_series_approximation(x: f32) -> f32 {
    // idea from https://stackoverflow.com/a/44232045/
    // modified to not be restricted to int range and only values of x above 1.0.
    // and got rid of most of the slow conversions,
    // should work for all positive values of x.

    //x may essentially be 1.0 but, as clippy notes, these kinds of
    //floating point comparisons can fail when the bit pattern is not the sames
    if abs(x - 1.0_f32) < f32::EPSILON {
        return 0.0_f32;
    }
    let x_less_than_1: bool = x < 1.0;
    // Note: we could use the fast inverse approximation here found in super::inv::inv_approx, but
    // the precision of such an approximation is assumed not good enough.
    let x_working: f32 = if x_less_than_1 { 1.0 / x } else { x };
    //according to the SO post ln(x) = ln((2^n)*y)= ln(2^n) + ln(y) = ln(2) * n + ln(y)
    //get exponent value
    let base2_exponent: u32 = extract_exponent_value(x_working) as u32;
    let divisor: f32 = f32::from_bits(x_working.to_bits() & EXPONENT_MASK);
    //supposedly normalizing between 1.0 and 2.0
    let x_working: f32 = x_working / divisor;
    //approximate polynomial generated from maple in the post using Remez Algorithm:
    //https://en.wikipedia.org/wiki/Remez_algorithm
    let ln_1to2_polynomial: f32 = -1.741_793_9_f32
        + (2.821_202_6_f32
            + (-1.469_956_8_f32 + (0.447_179_55_f32 - 0.056_570_851_f32 * x_working) * x_working)
                * x_working)
            * x_working;
    // ln(2) * n + ln(y)
    let result: f32 = (base2_exponent as f32) * f32::consts::LN_2 + ln_1to2_polynomial;
    if x_less_than_1 {
        -result
    } else {
        result
    }
}

fn log10_ln_approx(x: f32) -> f32 {
    //using change of base log10(x) = ln(x)/ln(10)
    let ln10_recip = f32::consts::LOG10_E;
    let fract_base_ln = ln10_recip;
    let value_ln = ln_1to2_series_approximation(x);
    value_ln * fract_base_ln
}

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
            conv = 20f32 * log10_ln_approx(conv / 0.00002f32);
            conv as u8
        });
        max
    }
}

impl<'a, P: gpio::Pin> SoundPressure<'a> for AdcMicrophone<'a, P> {
    fn read_sound_pressure(&self) -> ReturnCode {
        if self.state.get() == State::Idle {
            // self.enable_pin.map (|pin| pin.set ());
            self.state.set(State::ReadingSPL);
            self.spl_pos.set(0);
            self.adc.sample();
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }

    fn set_client(&self, client: &'a dyn SoundPressureClient) {
        self.spl_client.set(client);
    }

    fn enable(&self) -> ReturnCode {
        self.enable_pin.map(|pin| pin.set());
        ReturnCode::SUCCESS
    }

    fn disable(&self) -> ReturnCode {
        self.enable_pin.map(|pin| pin.clear());
        ReturnCode::SUCCESS
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
                    self.adc.sample();
                    false
                } else {
                    self.state.set(State::Idle);
                    true
                }
            }) {
                // self.enable_pin.map (|pin| pin.clear ());
                let spl = self.compute_spl();
                self.spl_client.map(|client| client.callback(spl));
            }
        }
    }
}
