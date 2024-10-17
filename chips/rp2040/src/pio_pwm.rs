// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2024.
//
// Author: Radu Matei <radu.matei.05.21@gmail.com>

//! Programmable Input Output (PIO) hardware test file.
use crate::clocks::{self};
use crate::gpio::RPGpio;
use crate::pio::{PIONumber, Pio, SMNumber, StateMachineConfiguration};

use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{hil, ErrorCode};

pub struct PioPwm<'a> {
    clocks: OptionalCell<&'a clocks::Clocks>,
    pio: TakeCell<'a, Pio>,
}

impl<'a> PioPwm<'a> {
    pub fn new(pio: &'a mut Pio) -> Self {
        Self {
            clocks: OptionalCell::empty(),
            pio: TakeCell::new(pio),
        }
    }

    pub fn set_clocks(&self, clocks: &'a clocks::Clocks) {
        self.clocks.set(clocks);
    }
}

impl<'a> hil::pwm::Pwm for PioPwm<'a> {
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
            pio.add_program(&path);
            let mut custom_config = StateMachineConfiguration::default();

            let pin_nr = *pin as u32;
            custom_config.div_frac = 0;
            custom_config.div_int = 1;
            custom_config.side_set_base = pin_nr;
            custom_config.side_set_bit_count = 2;
            custom_config.side_set_opt_enable = true;
            custom_config.side_set_pindirs = false;
            let max_freq = self.get_maximum_frequency_hz();
            let pwm_period = (max_freq / frequency_hz) as u32;
            let sm_number = SMNumber::SM0;
            let duty_cycle = duty_cycle_percentage as u32;
            pio.pwm_program_init(
                PIONumber::PIO0,
                sm_number,
                pin_nr,
                pwm_period,
                &custom_config,
            );
            pio.sm_put_blocking(
                sm_number,
                pwm_period * duty_cycle / (self.get_maximum_duty_cycle()) as u32,
            );
        });

        Ok(())
    }

    fn stop(&self, _pin: &Self::Pin) -> Result<(), ErrorCode> {
        self.pio.map(|pio| pio.clear_instr_registers());
        Ok(())
    }

    fn get_maximum_duty_cycle(&self) -> usize {
        // being a percentage, max duty cycle is 100
        10000
    }

    fn get_maximum_frequency_hz(&self) -> usize {
        self.clocks
            .unwrap_or_panic()
            .get_frequency(clocks::Clock::System) as usize
    }
}
