// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2024.
//
// Author: Radu Matei <radu.matei.05.21@gmail.com>

//! Programmable Input Output (PIO) hardware test file.
use crate::clocks::{self};
use crate::gpio::{RPGpio, RPGpioPin};
use crate::pio::{Pio, SMNumber, StateMachineConfiguration};
use enum_primitive::cast::FromPrimitive;

use kernel::utilities::cells::TakeCell;
use kernel::{ErrorCode, hil};

pub struct PioPwm<'a> {
    clocks: &'a clocks::Clocks,
    pio: TakeCell<'a, Pio>,
}

impl<'a> PioPwm<'a> {
    pub fn new(pio: &'a mut Pio, clocks: &'a clocks::Clocks) -> Self {
        Self {
            clocks,
            pio: TakeCell::new(pio),
        }
    }
}

impl hil::pwm::Pwm for PioPwm<'_> {
    type Pin = RPGpio;

    fn start(
        &self,
        pin: &Self::Pin,
        frequency_hz: usize,
        duty_cycle_percentage: usize,
    ) -> Result<(), ErrorCode> {
        // Ramps up the intensity of an LED using PWM.
        // .program pwm
        // .side_set 1 opt
        //     pull noblock    side 0 ; Pull from FIFO to OSR if available, else copy X to OSR.
        //     mov x, osr             ; Copy most-recently-pulled value back to scratch X
        //     mov y, isr             ; ISR contains PWM period. Y used as counter.
        // countloop:
        //     jmp x!=y noset         ; Set pin high if X == Y, keep the two paths length matched
        //     jmp skip        side 1
        // noset:
        //     nop                    ; Single dummy cycle to keep the two paths the same length
        // skip:
        //     jmp y-- countloop      ; Loop until Y hits 0, then pull a fresh PWM value from FIFO
        let path: [u8; 14] = [
            0x90, 0x80, 0xa0, 0x27, 0xa0, 0x46, 0x00, 0xa5, 0x18, 0x06, 0xa0, 0x42, 0x00, 0x83,
        ];

        self.pio.map(|pio| {
            pio.init();
            let _ = pio.add_program(Some(0), &path);
            let mut custom_config = StateMachineConfiguration::default();

            let pin_nr = *pin as u32;
            custom_config.div_frac = 0;
            custom_config.div_int = 1;
            custom_config.side_set_base = pin_nr;
            custom_config.side_set_bit_count = 2;
            custom_config.side_set_opt_enable = true;
            custom_config.side_set_pindirs = false;
            let max_freq = self.get_maximum_frequency_hz();
            let pwm_period = ((max_freq / frequency_hz) / 3) as u32;
            let sm_number = SMNumber::SM0;
            let duty_cycle = duty_cycle_percentage as u32;
            pwm_program_init(pio, sm_number, pin_nr, pwm_period, &custom_config);
            let _ = pio
                .sm(sm_number)
                .push_blocking(pwm_period * duty_cycle / (self.get_maximum_duty_cycle()) as u32);
        });

        Ok(())
    }

    fn stop(&self, _pin: &Self::Pin) -> Result<(), ErrorCode> {
        self.pio.map(|pio| pio.clear_instr_registers());
        Ok(())
    }

    fn get_maximum_duty_cycle(&self) -> usize {
        // being a percentage out of 10000, max duty cycle is 10000
        10000
    }

    // For the rp2040, this will always return 125_000_000. Watch out as any value above
    // 1_000_000 is not precise and WILL give modified frequency and duty cycle values.
    fn get_maximum_frequency_hz(&self) -> usize {
        self.clocks.get_frequency(clocks::Clock::System) as usize
    }
}

/// Load and start the PWM program on one state machine.
///
/// The period is pushed to the FIFO and then moved into the ISR by hand,
/// with a `pull` and an `out isr, 32` executed before the state machine is
/// released. The program reloads the period from the ISR on every wrap, so
/// it has to be there before the first one.
fn pwm_program_init(
    pio: &Pio,
    sm_number: SMNumber,
    pin: u32,
    pwm_period: u32,
    config: &StateMachineConfiguration,
) {
    let sm = pio.sm(sm_number);
    // "pull" command created by pioasm
    let pull_command = 0x8080_u16;
    // "out isr, 32" command created by pioasm
    let out_isr_32_command = 0x60c0_u16;
    sm.config(config);
    pio.gpio_init(&RPGpioPin::new(
        RPGpio::from_u32(pin).expect("GPIO pin must be 0 to 29"),
    ));
    sm.set_enabled(false);
    sm.set_pins_dirs(pin, 1, true);
    sm.set_side_set_pins(pin, 1, false, true);
    sm.init();
    let _ = sm.push_blocking(pwm_period);
    sm.exec(pull_command);
    sm.exec(out_isr_32_command);
    sm.set_enabled(true);
}
