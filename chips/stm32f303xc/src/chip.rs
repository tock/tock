// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Chip trait setup.

use core::fmt::Write;
use cortexm4f::{CortexM4F, CortexMVariant};
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::nvic;

pub struct Stm32f3xx<'a, I: InterruptService + 'a> {
    mpu: cortexm4f::mpu::MPU,
    userspace_kernel_boundary: cortexm4f::syscall::SysCall,
    interrupt_service: &'a I,
}

pub struct Stm32f3xxDefaultPeripherals<'a> {
    pub adc1: crate::adc::Adc<'a>,
    pub dma: crate::dma::Dma1<'a>,
    pub exti: &'a crate::exti::Exti<'a>,
    pub flash: crate::flash::Flash,
    pub i2c1: crate::i2c::I2C<'a>,
    pub spi1: crate::spi::Spi<'a>,
    pub tim2: crate::tim2::Tim2<'a>,
    pub usart1: crate::usart::Usart<'a>,
    pub usart2: crate::usart::Usart<'a>,
    pub usart3: crate::usart::Usart<'a>,
    pub gpio_ports: crate::gpio::GpioPorts<'a>,
    pub watchdog: crate::wdt::WindoWdg<'a>,
}

impl<'a> Stm32f3xxDefaultPeripherals<'a> {
    pub fn new(rcc: &'a crate::rcc::Rcc, exti: &'a crate::exti::Exti<'a>) -> Self {
        Self {
            adc1: crate::adc::Adc::new(rcc),
            dma: crate::dma::Dma1::new(rcc),
            exti,
            flash: crate::flash::Flash::new(),
            i2c1: crate::i2c::I2C::new_i2c1(rcc),
            spi1: crate::spi::Spi::new_spi1(rcc),
            tim2: crate::tim2::Tim2::new(rcc),
            usart1: crate::usart::Usart::new_usart1(rcc),
            usart2: crate::usart::Usart::new_usart2(rcc),
            usart3: crate::usart::Usart::new_usart3(rcc),
            gpio_ports: crate::gpio::GpioPorts::new(rcc, exti),
            watchdog: crate::wdt::WindoWdg::new(rcc),
        }
    }

    // Setup any circular dependencies and register deferred calls
    pub fn setup_circular_deps(&'static self) {
        self.gpio_ports.setup_circular_deps();

        kernel::deferred_call::DeferredCallClient::register(&self.flash);
        kernel::deferred_call::DeferredCallClient::register(&self.usart1);
        kernel::deferred_call::DeferredCallClient::register(&self.usart2);
        kernel::deferred_call::DeferredCallClient::register(&self.usart3);
    }
}

impl InterruptService for Stm32f3xxDefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            nvic::USART1 => self.usart1.handle_interrupt(),
            nvic::USART2 => self.usart2.handle_interrupt(),
            nvic::USART3 => self.usart3.handle_interrupt(),

            nvic::TIM2 => self.tim2.handle_interrupt(),

            nvic::SPI1 => self.spi1.handle_interrupt(),

            nvic::FLASH => self.flash.handle_interrupt(),

            nvic::I2C1_EV => self.i2c1.handle_event(),
            nvic::I2C1_ER => self.i2c1.handle_error(),
            nvic::ADC1_2 => self.adc1.handle_interrupt(),

            nvic::EXTI0 => self.exti.handle_interrupt(),
            nvic::EXTI1 => self.exti.handle_interrupt(),
            nvic::EXTI2 => self.exti.handle_interrupt(),
            nvic::EXTI3 => self.exti.handle_interrupt(),
            nvic::EXTI4 => self.exti.handle_interrupt(),
            nvic::EXTI9_5 => self.exti.handle_interrupt(),
            nvic::EXTI15_10 => self.exti.handle_interrupt(),
            _ => return false,
        }
        true
    }
}

impl<'a, I: InterruptService + 'a> Stm32f3xx<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm4f::mpu::new(),
            userspace_kernel_boundary: cortexm4f::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

impl<'a, I: InterruptService + 'a> Chip for Stm32f3xx<'a, I> {
    type MPU = cortexm4f::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4f::syscall::SysCall;
    type ThreadIdProvider = cortexm4f::thread_id::CortexMThreadIdProvider;

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = cortexm4f::nvic::next_pending() {
                if !self.interrupt_service.service_interrupt(interrupt) {
                    panic!("unhandled interrupt {}", interrupt);
                }
                let n = cortexm4f::nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm4f::nvic::has_pending() }
    }

    fn mpu(&self) -> &cortexm4f::mpu::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &cortexm4f::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm4f::scb::unset_sleepdeep();
            cortexm4f::support::wfi();
        }
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4f::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, write: &mut dyn Write) {
        CortexM4F::print_cortexm_state(write);
    }
}
