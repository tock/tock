// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Example peripheral collections for the nRF52 series of MCUs.

/// This struct, when initialized, instantiates all peripheral drivers for the nrf52.
///
/// If a board wishes to use only a subset of these peripherals, this
/// should not be used or imported, and a modified version should be
/// constructed manually in main.rs.
pub struct Nrf52DefaultPeripherals<'a> {
    pub acomp: crate::acomp::Comparator<'a>,
    pub ecb: crate::aes::AesECB<'a>,
    pub pwr_clk: crate::power::Power<'a>,
    pub ble_radio: crate::ble_radio::Radio<'a>,
    pub trng: crate::trng::Trng<'a>,
    pub rtc: crate::rtc::Rtc<'a>,
    pub temp: crate::temperature::Temp<'a>,
    pub timer0: crate::timer::TimerAlarm<'a>,
    pub timer1: crate::timer::TimerAlarm<'a>,
    pub timer2: crate::timer::Timer,
    pub uarte0: crate::uart::Uarte<'a>,
    pub spim0: crate::spi::SPIM<'a>,
    pub twi1: crate::i2c::TWI<'a>,
    pub spim2: crate::spi::SPIM<'a>,
    pub adc: crate::adc::Adc<'a>,
    pub nvmc: crate::nvmc::Nvmc,
    pub clock: crate::clock::Clock,
    pub pwm0: crate::pwm::Pwm,
}

impl Nrf52DefaultPeripherals<'_> {
    pub fn new() -> Self {
        Self {
            acomp: crate::acomp::Comparator::new(),
            ecb: crate::aes::AesECB::new(),
            pwr_clk: crate::power::Power::new(),
            ble_radio: crate::ble_radio::Radio::new(),
            trng: crate::trng::Trng::new(),
            rtc: crate::rtc::Rtc::new(),
            temp: crate::temperature::Temp::new(),
            timer0: crate::timer::TimerAlarm::new(0),
            timer1: crate::timer::TimerAlarm::new(1),
            timer2: crate::timer::Timer::new(2),
            uarte0: crate::uart::Uarte::new(crate::uart::UARTE0_BASE),
            spim0: crate::spi::SPIM::new(0),
            twi1: crate::i2c::TWI::new_twi1(),
            spim2: crate::spi::SPIM::new(2),
            // Default to 3.3 V VDD reference.
            adc: crate::adc::Adc::new(3300),
            nvmc: crate::nvmc::Nvmc::new(),
            clock: crate::clock::Clock::new(),
            pwm0: crate::pwm::Pwm::new(),
        }
    }
    // Necessary for setting up circular dependencies
    pub fn init(&'static self) {
        kernel::deferred_call::DeferredCallClient::register(&self.nvmc);
    }
}
impl kernel::platform::chip::InterruptService for Nrf52DefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            crate::peripheral_interrupts::COMP => self.acomp.handle_interrupt(),
            crate::peripheral_interrupts::ECB => self.ecb.handle_interrupt(),
            crate::peripheral_interrupts::POWER_CLOCK => self.pwr_clk.handle_interrupt(),
            crate::peripheral_interrupts::RADIO => match self.ble_radio.is_enabled() {
                false => (),
                true => self.ble_radio.handle_interrupt(),
            },
            crate::peripheral_interrupts::RNG => self.trng.handle_interrupt(),
            crate::peripheral_interrupts::RTC1 => self.rtc.handle_interrupt(),
            crate::peripheral_interrupts::TEMP => self.temp.handle_interrupt(),
            crate::peripheral_interrupts::TIMER0 => self.timer0.handle_interrupt(),
            crate::peripheral_interrupts::TIMER1 => self.timer1.handle_interrupt(),
            crate::peripheral_interrupts::TIMER2 => self.timer2.handle_interrupt(),
            crate::peripheral_interrupts::UART0 => self.uarte0.handle_interrupt(),
            crate::peripheral_interrupts::SPI0_TWI0 => self.spim0.handle_interrupt(),
            crate::peripheral_interrupts::SPI1_TWI1 => self.twi1.handle_interrupt(),
            crate::peripheral_interrupts::SPIM2_SPIS2_SPI2 => self.spim2.handle_interrupt(),
            crate::peripheral_interrupts::ADC => self.adc.handle_interrupt(),
            _ => return false,
        }
        true
    }
}
