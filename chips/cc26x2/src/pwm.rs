use enum_primitive::cast::FromPrimitive;
use gpt;
use prcm;

enum_from_primitive!{
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Timer {
    GPT0A = 0,
    GPT0B = 1,
    GPT1A = 2,
    GPT1B = 3,
    GPT2A = 4,
    GPT2B = 5,
    GPT3A = 6,
    GPT3B = 7
}
}

enum_from_primitive!{
#[derive(Debug, PartialEq, Clone, Copy)]
enum Gpt {
    GPT0 = 0,
    GPT1 = 1,
    GPT2 = 2,
    GPT3 = 3,
}
}

use kernel::common::registers::{Field, ReadWrite};
// this struct helps group together 16-bit timers
pub struct Channel<'a> {
    gpt: Gpt,
    mode: &'a ReadWrite<u32, gpt::Mode::Register>,
    prescale: &'a ReadWrite<u32, gpt::Prescale::Register>,
    prescale_match: &'a ReadWrite<u32, gpt::Prescale::Register>,
    timer_load: &'a ReadWrite<u32, gpt::Value32::Register>,
    timer_match: &'a ReadWrite<u32, gpt::Value32::Register>,
    ctl_enable_field: &'a Field<u32, gpt::Ctl::Register>,
    ctl_output_invert_field: &'a Field<u32, gpt::Ctl::Register>,
}

impl<'a> Channel<'a> {
    pub fn new(timer: Timer) -> Channel<'a> {
        let gpt;
        match timer {
            Timer::GPT0A => gpt = Gpt::GPT0,
            Timer::GPT0B => gpt = Gpt::GPT0,
            Timer::GPT1A => gpt = Gpt::GPT1,
            Timer::GPT1B => gpt = Gpt::GPT1,
            Timer::GPT2A => gpt = Gpt::GPT2,
            Timer::GPT2B => gpt = Gpt::GPT2,
            Timer::GPT3A => gpt = Gpt::GPT3,
            Timer::GPT3B => gpt = Gpt::GPT3,
        }

        match timer {
            Timer::GPT0A | Timer::GPT1A | Timer::GPT2A | Timer::GPT3A => Channel {
                gpt,
                mode: &gpt::GPT[gpt as usize].timer_a_mode,
                prescale: &gpt::GPT[gpt as usize].timer_a_prescale,
                prescale_match: &gpt::GPT[gpt as usize].timer_a_prescale_match,
                timer_load: &gpt::GPT[gpt as usize].timer_a_load,
                timer_match: &gpt::GPT[gpt as usize].timer_a_match,
                ctl_enable_field: &gpt::Ctl::TIMER_A_EN,
                ctl_output_invert_field: &gpt::Ctl::TIMER_A_PWM_OUTPUT_INVERT,
            },
            Timer::GPT0B | Timer::GPT1B | Timer::GPT2B | Timer::GPT3B => Channel {
                gpt,
                mode: &gpt::GPT[gpt as usize].timer_b_mode,
                prescale: &gpt::GPT[gpt as usize].timer_b_prescale,
                prescale_match: &gpt::GPT[gpt as usize].timer_b_prescale_match,
                timer_load: &gpt::GPT[gpt as usize].timer_b_load,
                timer_match: &gpt::GPT[gpt as usize].timer_b_match,
                ctl_enable_field: &gpt::Ctl::TIMER_B_EN,
                ctl_output_invert_field: &gpt::Ctl::TIMER_B_PWM_OUTPUT_INVERT,
            },
        }
    }

    pub fn set_prescalar(&self, scalar: u8) {
        self.prescale.write(gpt::Prescale::RATIO.val(scalar as u32));
        self.prescale_match
            .write(gpt::Prescale::RATIO.val((scalar >> 1) as u32));
    }

    pub fn set_period(&self, period: u16) {
        self.timer_load.write(gpt::Value32::SET.val(period as u32));
    }

    pub fn set_on_period(&self, duty: u16) {
        self.timer_match.write(gpt::Value32::SET.val(duty as u32));
    }

    pub fn enable(&self, period: u16, on_period: u16) {
        prcm::Clock::enable_gpt(self.gpt as usize);

        // // 1. Ensure the timer is disabled (clear the TnEN bit) before making any changes.
        gpt::GPT[self.gpt as usize]
            .ctl
            .modify(self.ctl_enable_field.val(0));

        // // 2. Write the GPTM Configuration register (GPT:CFG) with a value of 0x0000 0004.
        // gpt::GPT[0].cfg.write(gpt::Cfg::BITS::_16);
        gpt::GPT[self.gpt as usize].cfg.write(gpt::Cfg::BITS::_16);

        // 3. In the GPTM Timer Mode register (GPT:TnMR), write the TnCMR field to 0x1 and write the TnMR field to 0x2.
        self.mode.write(
            gpt::Mode::MODE::PERIODIC
                + gpt::Mode::COUNT_DIRECTION::UP
                + gpt::Mode::ALT_MODE::PWM
                + gpt::Mode::CAPTURE_MODE::EDGE_COUNT
                + gpt::Mode::LEGACY_OP::DISABLE
                + gpt::Mode::PWM_INT::ENABLE
                + gpt::Mode::REG_UPDATE_MODE::CYCLE,
        );

        // // 5. If a prescaler is to be used, write the prescale value to the GPTM Timer n Prescale register (GPT:TnPR).
        self.set_prescalar(0);

        // // 7. Load the timer start value into the GPTM Timer n Interval Load register (GPT:TnILR).
        self.set_period(period);

        // // 8. Load the GPTM Timer n Match register (GPT:TnMATCHR) with the match value.
        self.set_on_period(on_period);

        // // enable the PWM and invert it so that on_period 255 ~= 100% duty cyle
        gpt::GPT[self.gpt as usize]
            .ctl
            .modify(self.ctl_enable_field.val(1) + self.ctl_output_invert_field.val(1));
    }
}
