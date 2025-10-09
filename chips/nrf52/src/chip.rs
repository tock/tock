// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::fmt::Write;
use cortexm4f::{nvic, CortexM4F, CortexMVariant};
use kernel::platform::chip::InterruptService;
use kernel::utilities::StaticRef;

//
// Peripheral Registers Instantiations
//

const AESECB_BASE: StaticRef<crate::aes::AesEcbRegisters> =
    unsafe { StaticRef::new(0x4000E000 as *const crate::aes::AesEcbRegisters) };

const RTC1_BASE: StaticRef<crate::rtc::RtcRegisters> =
    unsafe { StaticRef::new(0x40011000 as *const crate::rtc::RtcRegisters) };

const TEMP_BASE: StaticRef<crate::temperature::TempRegisters> =
    unsafe { StaticRef::new(0x4000C000 as *const crate::temperature::TempRegisters) };

const TIMER0_BASE: StaticRef<crate::timer::TimerRegisters> =
    unsafe { StaticRef::new(0x40008000 as *const crate::timer::TimerRegisters) };

const TIMER1_BASE: StaticRef<crate::timer::TimerRegisters> =
    unsafe { StaticRef::new(0x40009000 as *const crate::timer::TimerRegisters) };

const TIMER2_BASE: StaticRef<crate::timer::TimerRegisters> =
    unsafe { StaticRef::new(0x4000A000 as *const crate::timer::TimerRegisters) };

const RNG_BASE: StaticRef<crate::trng::RngRegisters> =
    unsafe { StaticRef::new(0x4000D000 as *const crate::trng::RngRegisters) };

pub struct NRF52<'a, I: InterruptService + 'a> {
    mpu: cortexm4f::mpu::MPU,
    userspace_kernel_boundary: cortexm4f::syscall::SysCall,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService + 'a> NRF52<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm4f::mpu::new(),
            userspace_kernel_boundary: cortexm4f::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

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
    pub fn new(aes_ecb_buffer: &'static mut [u8; 48]) -> Self {
        Self {
            acomp: crate::acomp::Comparator::new(),
            ecb: crate::aes::AesECB::new(AESECB_BASE, aes_ecb_buffer),
            pwr_clk: crate::power::Power::new(),
            ble_radio: crate::ble_radio::Radio::new(),
            trng: crate::trng::Trng::new(RNG_BASE),
            rtc: crate::rtc::Rtc::new(RTC1_BASE),
            temp: crate::temperature::Temp::new(TEMP_BASE),
            timer0: crate::timer::TimerAlarm::new(TIMER0_BASE),
            timer1: crate::timer::TimerAlarm::new(TIMER1_BASE),
            timer2: crate::timer::Timer::new(TIMER2_BASE),
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

impl<'a, I: InterruptService + 'a> kernel::platform::chip::Chip for NRF52<'a, I> {
    type MPU = cortexm4f::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4f::syscall::SysCall;
    type ThreadIdProvider = cortexm4f::thread_id::CortexMThreadIdProvider;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = nvic::next_pending() {
                if !self.interrupt_service.service_interrupt(interrupt) {
                    panic!("unhandled interrupt {}", interrupt);
                }
                let n = nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { nvic::has_pending() }
    }

    fn sleep(&self) {
        unsafe {
            cortexm4f::support::wfi();
        }
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4f::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(&self, write: &mut dyn Write) {
        CortexM4F::print_cortexm_state(write);
    }
}
