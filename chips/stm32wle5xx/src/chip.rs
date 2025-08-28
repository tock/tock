// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Chip trait setup.

use core::fmt::Write;
use cortexm4::{CortexM4, CortexMVariant};
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::chip_specific::chip_specs::ChipSpecs as ChipSpecsTrait;
use crate::nvic;

pub struct Stm32wle5xx<'a, I: InterruptService + 'a> {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    interrupt_service: &'a I,
}

pub struct Stm32wle5xxDefaultPeripherals<'a, ChipSpecs> {
    pub clocks: &'a crate::clocks::Clocks<'a, ChipSpecs>,
    pub gpio_ports: crate::gpio::GpioPorts<'a>,
    pub usart1: crate::usart::Usart<'a>,
    pub usart2: crate::usart::Usart<'a>,
    pub tim2: crate::tim2::Tim2<'a>,
    // pub i2c1: crate::i2c::I2C<'a>,
    pub i2c2: crate::i2c::I2C<'a>,
}

impl<'a, ChipSpecs: ChipSpecsTrait> Stm32wle5xxDefaultPeripherals<'a, ChipSpecs> {
    pub fn new(clocks: &'a crate::clocks::Clocks<'a, ChipSpecs>) -> Self {
        Self {
            clocks,
            gpio_ports: crate::gpio::GpioPorts::new(clocks),
            usart1: crate::usart::Usart::new_usart1(clocks),
            usart2: crate::usart::Usart::new_usart2(clocks),
            tim2: crate::tim2::Tim2::new(clocks),
            // i2c1: crate::i2c::I2C::new(clocks),
            i2c2: crate::i2c::I2C::new(clocks),
        }
    }

    // Setup any circular dependencies and register deferred calls
    pub fn setup_circular_deps(&'static self) {
        self.gpio_ports.setup_circular_deps();
        kernel::deferred_call::DeferredCallClient::register(&self.usart1);
        kernel::deferred_call::DeferredCallClient::register(&self.usart2);
    }
}

impl<ChipSpecs: ChipSpecsTrait> InterruptService for Stm32wle5xxDefaultPeripherals<'_, ChipSpecs> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            nvic::USART1 => self.usart1.handle_interrupt(),
            nvic::USART2 => self.usart2.handle_interrupt(),
            nvic::TIM2 => self.tim2.handle_interrupt(),

            // nvic::I2C1_EV => self.i2c1.handle_event(),
            // nvic::I2C1_ER => self.i2c1.handle_error(),
            nvic::I2C2_EV => self.i2c2.handle_event(),
            nvic::I2C2_ER => self.i2c2.handle_error_event(),

            _ => return false,
        }
        true
    }
}

impl<'a, I: InterruptService + 'a> Stm32wle5xx<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm4::mpu::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

impl<'a, I: InterruptService + 'a> Chip for Stm32wle5xx<'a, I> {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type ThreadIdProvider = cortexm4::thread_id::CortexMThreadIdProvider;

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(interrupt) = cortexm4f::nvic::next_pending() {
                    if !self.interrupt_service.service_interrupt(interrupt) {
                        panic!("unhandled interrupt {}", interrupt);
                    }

                    let n = cortexm4::nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm4f::nvic::has_pending() }
    }

    fn mpu(&self) -> &cortexm4::mpu::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &cortexm4::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4::support::with_interrupts_disabled(f)
    }

    fn sleep(&self) {
        unsafe {
            cortexm4::scb::unset_sleepdeep();
            cortexm4::support::wfi();
        }
    }

    unsafe fn print_state(_this: Option<&Self>, write: &mut dyn Write) {
        CortexM4::print_cortexm_state(write);
    }
}
