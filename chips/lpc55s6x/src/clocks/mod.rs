// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

pub mod syscon;

use self::syscon::SysconRegisters;
use kernel::utilities::{registers::interfaces::ReadWriteable, StaticRef};

pub const SYSCON_BASE: StaticRef<SysconRegisters> =
    unsafe { StaticRef::new(0x40000000 as *const SysconRegisters) };

pub enum Peripheral {
    Flexcomm0,
    Flexcomm1,
    Flexcomm2,
    Flexcomm3,
    Flexcomm4,
    Gpio0,
    Gpio1,
    Dma0,
}

#[repr(u8)]
pub enum FrgId {
    Frg0 = 0,
    Frg1 = 1,
    Frg2 = 2,
    Frg3 = 3,
    Frg4 = 4,
    Frg5 = 5,
    Frg6 = 6,
    Frg7 = 7,
}

#[derive(Copy, Clone)]
pub enum FrgClockSource {
    MainClock,
    SystemPll,
    Fro12Mhz,
    Fro96Mhz,
    Fro1Mhz,
    Mclk,
    Osc32Khz,
    NoClock,
}

pub struct Clock {
    syscon: &'static SysconRegisters,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            syscon: &SYSCON_BASE,
        }
    }

    pub fn start_gpio_clocks(&self) {
        self.syscon.ahbclkctrl0.modify(
            syscon::AHBCLKCTRL0::SRAM_CTRL1::SET
                + syscon::AHBCLKCTRL0::SRAM_CTRL2::SET
                + syscon::AHBCLKCTRL0::SRAM_CTRL3::SET
                + syscon::AHBCLKCTRL0::SRAM_CTRL4::SET
                + syscon::AHBCLKCTRL0::IOCON::SET
                + syscon::AHBCLKCTRL0::GPIO0::SET
                + syscon::AHBCLKCTRL0::GPIO1::SET
                + syscon::AHBCLKCTRL0::GPIO2::SET
                + syscon::AHBCLKCTRL0::GPIO3::SET
                + syscon::AHBCLKCTRL0::PINT::SET
                + syscon::AHBCLKCTRL0::MUX::SET,
        );
    }

    pub fn start_timer_clocks(&self) {
        self.syscon
            .ctimerclksel0
            .modify(syscon::CTIMERCLKSEL0::SEL::CLEAR);

        self.syscon
            .ahbclkctrl1
            .modify(syscon::AHBCLKCTRL1::TIMER0::SET);

        self.syscon.clkoutsel.modify(syscon::CLKOUTSEL::SEL::SET);
    }

    pub fn set_frg_clock_source(&self, frg_id: FrgId, source: FrgClockSource) {
        let sel_val = match source {
            FrgClockSource::MainClock => syscon::FCCLKSEL::SEL::MainClock,
            FrgClockSource::SystemPll => syscon::FCCLKSEL::SEL::SystemPLLDividedClock,
            FrgClockSource::Fro12Mhz => syscon::FCCLKSEL::SEL::FRO12MHzClock,
            FrgClockSource::Fro96Mhz => syscon::FCCLKSEL::SEL::FRO96MHzClock,
            FrgClockSource::Fro1Mhz => syscon::FCCLKSEL::SEL::FRO1MHzClock,
            FrgClockSource::Mclk => syscon::FCCLKSEL::SEL::MCLKClock,
            FrgClockSource::Osc32Khz => syscon::FCCLKSEL::SEL::Oscillator32KHzClock,
            FrgClockSource::NoClock => syscon::FCCLKSEL::SEL::NoClock,
        };

        match frg_id {
            FrgId::Frg0 => self.syscon.fcclksel0.modify(sel_val),
            FrgId::Frg1 => self.syscon.fcclksel1.modify(sel_val),
            FrgId::Frg2 => self.syscon.fcclksel2.modify(sel_val),
            FrgId::Frg3 => self.syscon.fcclksel3.modify(sel_val),
            FrgId::Frg4 => self.syscon.fcclksel4.modify(sel_val),
            FrgId::Frg5 => self.syscon.fcclksel5.modify(sel_val),
            FrgId::Frg6 => self.syscon.fcclksel6.modify(sel_val),
            FrgId::Frg7 => self.syscon.fcclksel7.modify(sel_val),
        }
    }

    pub fn get_frg_clock_frequency(&self, source: FrgClockSource) -> u32 {
        match source {
            FrgClockSource::Fro12Mhz => 12_000_000,
            FrgClockSource::Fro96Mhz => 96_000_000,
            FrgClockSource::Fro1Mhz => 1_000_000,
            FrgClockSource::Osc32Khz => 32_768,
            FrgClockSource::MainClock => 12_000_000, //not definitive, should check mainclksel
            FrgClockSource::SystemPll => 0,
            FrgClockSource::Mclk => 0,
            FrgClockSource::NoClock => 0,
        }
    }
}
