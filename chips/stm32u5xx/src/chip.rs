// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright OxidOS Automotive 2026.

use crate::dma::{ChannelId, Dma};
use crate::nvic::{
    EXTI13_IRQ, GPDMA1_CH0_IRQ, GPDMA1_CH10_IRQ, GPDMA1_CH11_IRQ, GPDMA1_CH12_IRQ, GPDMA1_CH13_IRQ,
    GPDMA1_CH14_IRQ, GPDMA1_CH15_IRQ, GPDMA1_CH1_IRQ, GPDMA1_CH2_IRQ, GPDMA1_CH3_IRQ,
    GPDMA1_CH4_IRQ, GPDMA1_CH5_IRQ, GPDMA1_CH6_IRQ, GPDMA1_CH7_IRQ, GPDMA1_CH8_IRQ, GPDMA1_CH9_IRQ,
    TIM2_IRQ, USART1_IRQ,
};
use core::fmt::Write;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

pub struct Stm32u5xx<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

pub struct Stm32u5xxDefaultPeripherals<'a> {
    pub tim2: &'a crate::tim::Tim2<'a>,
    pub usart1: &'a crate::usart::Usart<'a>,
    pub exti: &'a crate::exti::Exti<'a>,
    pub dma1: &'a Dma,
}

impl<'a> Stm32u5xxDefaultPeripherals<'a> {
    pub fn new(
        tim2: &'a crate::tim::Tim2<'a>,
        usart1: &'a crate::usart::Usart<'a>,
        exti: &'a crate::exti::Exti<'a>,
        dma1: &'a Dma,
    ) -> Self {
        Self {
            tim2,
            usart1,
            exti,
            dma1,
        }
    }
}

impl InterruptService for Stm32u5xxDefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            TIM2_IRQ => {
                // TIM2
                self.tim2.handle_interrupt();
                true
            }
            USART1_IRQ => {
                // USART1
                self.usart1.handle_interrupt();
                true
            }
            EXTI13_IRQ => {
                // EXTI13 (Button)
                self.exti.handle_interrupt(crate::exti::LineId::Line13);
                true
            }
            // Route all 16 GPDMA1 Channels to the DMA manager
            GPDMA1_CH0_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel00);
                true
            }
            GPDMA1_CH1_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel01);
                true
            }
            GPDMA1_CH2_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel02);
                true
            }
            GPDMA1_CH3_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel03);
                true
            }
            GPDMA1_CH4_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel04);
                true
            }
            GPDMA1_CH5_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel05);
                true
            }
            GPDMA1_CH6_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel06);
                true
            }
            GPDMA1_CH7_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel07);
                true
            }
            GPDMA1_CH8_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel08);
                true
            }
            GPDMA1_CH9_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel09);
                true
            }
            GPDMA1_CH10_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel10);
                true
            }
            GPDMA1_CH11_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel11);
                true
            }
            GPDMA1_CH12_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel12);
                true
            }
            GPDMA1_CH13_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel13);
                true
            }
            GPDMA1_CH14_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel14);
                true
            }
            GPDMA1_CH15_IRQ => {
                self.dma1.handle_interrupt(ChannelId::Channel15);
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
