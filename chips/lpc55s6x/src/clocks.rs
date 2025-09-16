// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use crate::syscon::{self, SysconRegisters};
use kernel::utilities::{
    registers::interfaces::{ReadWriteable, Writeable},
    StaticRef,
};

// const CLKCTL_BASE: StaticRef<ClkctlRegisters> =
//     unsafe { StaticRef::new(0x4000_2000 as *const ClkctlRegisters) };

// const PMC_BASE: StaticRef<PmcRegisters> =
//     unsafe { StaticRef::new(0x5000_1000 as *const PmcRegisters) };

// const ANACTRL_BASE: StaticRef<AnactrlRegisters> =
//     unsafe { StaticRef::new(0x5000_3000 as *const AnactrlRegisters) };

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

    pub fn set_frg_clock_source(&self, frg_id: u32, source: FrgClockSource) {
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
            0 => self.syscon.fcclksel0.write(sel_val),
            1 => self.syscon.fcclksel1.write(sel_val),
            2 => self.syscon.fcclksel2.write(sel_val),
            3 => self.syscon.fcclksel3.write(sel_val),
            4 => self.syscon.fcclksel4.write(sel_val),
            5 => self.syscon.fcclksel5.write(sel_val),
            6 => self.syscon.fcclksel6.write(sel_val),
            7 => self.syscon.fcclksel7.write(sel_val),
            _ => {}
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

    pub fn setup_uart_clock(&self, flexcomm_id: u32, frg_source: FrgClockSource) {
        // Enabling the bus clock for the peripheral
        match flexcomm_id {
            0 => self
                .syscon
                .ahbclkctrl1
                .modify(syscon::AHBCLKCTRL1::FC0::SET),
            1 => self
                .syscon
                .ahbclkctrl1
                .modify(syscon::AHBCLKCTRL1::FC1::SET),
            2 => self
                .syscon
                .ahbclkctrl1
                .modify(syscon::AHBCLKCTRL1::FC2::SET),
            3 => self
                .syscon
                .ahbclkctrl1
                .modify(syscon::AHBCLKCTRL1::FC3::SET),
            4 => self
                .syscon
                .ahbclkctrl1
                .modify(syscon::AHBCLKCTRL1::FC4::SET),
            5 => self
                .syscon
                .ahbclkctrl1
                .modify(syscon::AHBCLKCTRL1::FC5::SET),
            6 => self
                .syscon
                .ahbclkctrl1
                .modify(syscon::AHBCLKCTRL1::FC6::SET),
            7 => self
                .syscon
                .ahbclkctrl1
                .modify(syscon::AHBCLKCTRL1::FC7::SET),
            _ => return,
        }

        // Setting the clock source for the Fractional Rate Divider
        self.set_frg_clock_source(flexcomm_id, frg_source);
    }
}
