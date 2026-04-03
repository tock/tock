// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::fmt::Write;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

pub struct Stm32u5xx<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

pub struct Stm32u5xxDefaultPeripherals<'a> {
    // Peripherals will go here
    pub tim2: &'a crate::tim::Tim2<'a>,
    pub usart1: &'a crate::usart::Usart<'a>,
    pub exti: &'a crate::exti::Exti<'a>,
}

impl<'a> Stm32u5xxDefaultPeripherals<'a> {
    pub fn new(
        tim2: &'a crate::tim::Tim2<'a>,
        usart1: &'a crate::usart::Usart<'a>,
        exti: &'a crate::exti::Exti<'a>,
    ) -> Self {
        Self { tim2, usart1, exti }
    }
}

impl InterruptService for Stm32u5xxDefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            45 => {
                // TIM2
                self.tim2.handle_interrupt();
                true
            }
            61 => {
                // USART1
                self.usart1.handle_interrupt();
                true
            }
            24 => {
                // EXTI13 (Button)
                self.exti.handle_interrupt(13);
                true
            }
            29 => {
                // GPDMA1 Channel 0 (USART1 TX Complete)
                self.usart1.handle_dma_interrupt(true);
                true
            }
            30 => {
                // GPDMA1 Channel 1 (USART1 RX Complete)
                self.usart1.handle_dma_interrupt(false);
                true
            }
            _ => false,
        }
    }
}

impl<'a, I: InterruptService + 'a> Stm32u5xx<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm33::mpu::new::<8>(),
            userspace_kernel_boundary: cortexm33::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

impl<'a, I: InterruptService + 'a> Chip for Stm32u5xx<'a, I> {
    type MPU = cortexm33::mpu::MPU<8>;
    type UserspaceKernelBoundary = cortexm33::syscall::SysCall;
    type ThreadIdProvider = cortexm33::thread_id::CortexMThreadIdProvider;

    fn service_pending_interrupts(&self) {
        unsafe {
            while let Some(interrupt) = cortexm33::nvic::next_pending() {
                if !self.interrupt_service.service_interrupt(interrupt) {
                    panic!("unhandled interrupt {}", interrupt);
                }

                let n = cortexm33::nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm33::nvic::has_pending() }
    }

    fn mpu(&self) -> &cortexm33::mpu::MPU<8> {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &cortexm33::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm33::scb::unset_sleepdeep();
            cortexm33::support::wfi();
        }
    }

    unsafe fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm33::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, write: &mut dyn Write) {
        let _ = write.write_str("Cortex-M33 state\n");
    }
}
