// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Author: Kamil Duljas <kamil.duljas@gmail.com>

use core::fmt::Write;
use cortexm4f::{CortexM4F, CortexMVariant};
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

use crate::nvic;

use crate::chip_specific::chip_specs::ChipSpecs as ChipSpecsTrait;

pub struct Stm32l4xx<'a, I: InterruptService + 'a> {
    mpu: cortexm4f::mpu::MPU,
    userspace_kernel_boundary: cortexm4f::syscall::SysCall,
    interrupt_service: &'a I,
}

pub struct Stm32l4xxDefaultPeripherals<'a, ChipSpecs> {
    pub gpio_ports: crate::gpio::GpioPorts<'a>,
    pub clocks: &'a crate::clocks::Clocks<'a, ChipSpecs>,
    pub flash: crate::flash::Flash<ChipSpecs>,
    pub exti: &'a crate::exti::Exti<'a>,
    pub usart1: crate::usart::Usart<'a>,
    pub usart2: crate::usart::Usart<'a>,
    pub usart3: crate::usart::Usart<'a>,
}

impl<'a, ChipSpecs: ChipSpecsTrait> Stm32l4xxDefaultPeripherals<'a, ChipSpecs> {
    pub fn new(
        clocks: &'a crate::clocks::Clocks<'a, ChipSpecs>,
        exti: &'a crate::exti::Exti<'a>,
    ) -> Self {
        let pwr = crate::pwr::Pwr::new();
        Self {
            clocks,
            gpio_ports: crate::gpio::GpioPorts::new(clocks, exti),
            flash: crate::flash::Flash::new(pwr),
            exti,
            usart1: crate::usart::Usart::new_usart1(clocks),
            usart2: crate::usart::Usart::new_usart2(clocks),
            usart3: crate::usart::Usart::new_usart3(clocks),
        }
    }

    pub fn setup_circular_deps(&'static self) {
        self.clocks.set_flash(&self.flash);
        self.gpio_ports.setup_circular_deps();

        kernel::deferred_call::DeferredCallClient::register(&self.usart1);
        kernel::deferred_call::DeferredCallClient::register(&self.usart2);
        kernel::deferred_call::DeferredCallClient::register(&self.usart3);
    }
}

impl<ChipSpecs: ChipSpecsTrait> InterruptService for Stm32l4xxDefaultPeripherals<'_, ChipSpecs> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            nvic::USART1 => self.usart1.handle_interrupt(),
            nvic::USART2 => self.usart2.handle_interrupt(),
            nvic::USART3 => self.usart3.handle_interrupt(),
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

impl<'a, I: InterruptService + 'a> Stm32l4xx<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm4f::mpu::new(),
            userspace_kernel_boundary: cortexm4f::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

impl<'a, I: InterruptService + 'a> Chip for Stm32l4xx<'a, I> {
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
