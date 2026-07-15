// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright OxidOS Automotive 2026.

use crate::dac;
use crate::dma::{ChannelId, Dma};
use crate::exti;
use crate::gpio;
use crate::i2c;
use crate::nvic::{
    EXTI0_IRQ, EXTI1_IRQ, EXTI2_IRQ, EXTI3_IRQ, EXTI4_IRQ, EXTI5_IRQ, EXTI6_IRQ, EXTI7_IRQ,
    EXTI8_IRQ, EXTI9_IRQ, EXTI10_IRQ, EXTI11_IRQ, EXTI12_IRQ, EXTI13_IRQ, EXTI14_IRQ, EXTI15_IRQ,
    GPDMA1_CH0_IRQ, GPDMA1_CH1_IRQ, GPDMA1_CH2_IRQ, GPDMA1_CH3_IRQ, GPDMA1_CH4_IRQ, GPDMA1_CH5_IRQ,
    GPDMA1_CH6_IRQ, GPDMA1_CH7_IRQ, GPDMA1_CH8_IRQ, GPDMA1_CH9_IRQ, GPDMA1_CH10_IRQ,
    GPDMA1_CH11_IRQ, GPDMA1_CH12_IRQ, GPDMA1_CH13_IRQ, GPDMA1_CH14_IRQ, GPDMA1_CH15_IRQ,
    I2C1_ER_IRQ, I2C1_EV_IRQ, TIM2_IRQ, USART1_IRQ,
};
use crate::rcc;
use crate::tim;
use crate::usart;

use core::fmt::Write;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

pub struct Stm32u5xx<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

pub struct Stm32u5xxDefaultPeripherals<'a> {
    pub rcc: rcc::Rcc,
    pub tim2: tim::Tim2<'a>,
    pub usart1: &'a usart::Usart<'a>,
    pub i2c1: i2c::I2c<'a>,
    pub exti: &'a exti::Exti<'a>,
    pub dma1: &'a Dma,
    pub gpio_a: gpio::Port<'a>,
    pub gpio_b: gpio::Port<'a>,
    pub gpio_c: gpio::Port<'a>,
    pub dac: dac::Dac,
}

fn enable_tim2_clock() {
    let rcc = rcc::Rcc::new(rcc::RCC_BASE);
    rcc.enable_tim2();
}

fn enable_dac1_clock() {
    let rcc = rcc::Rcc::new(rcc::RCC_BASE);
    rcc.enable_dac1();
}

impl<'a> Stm32u5xxDefaultPeripherals<'a> {
    pub fn new(usart1: &'a usart::Usart<'a>, exti: &'a exti::Exti<'a>, dma1: &'a Dma) -> Self {
        Self {
            rcc: rcc::Rcc::new(rcc::RCC_BASE),
            tim2: tim::Tim2::new(tim::TIM2_BASE, enable_tim2_clock),
            usart1,
            i2c1: i2c::I2c::new(i2c::I2C1_BASE),
            exti,
            dma1,
            gpio_a: gpio::Port::new(gpio::GPIO_A_BASE, exti, gpio::GpioPort::PortA),
            gpio_b: gpio::Port::new(gpio::GPIO_B_BASE, exti, gpio::GpioPort::PortB),
            gpio_c: gpio::Port::new(gpio::GPIO_C_BASE, exti, gpio::GpioPort::PortC),
            dac: dac::Dac::new(dac::DAC_BASE, enable_dac1_clock),
        }
    }

    pub fn init(&'static self) {
        // Power and Wires
        self.rcc.enable_dma1();
        self.rcc.enable_gpioa();
        self.rcc.enable_gpiob();
        self.rcc.enable_gpioc();
        self.rcc.enable_usart1();
        self.rcc.enable_syscfg();
        self.rcc.enable_i2c1();
        self.rcc.set_usart1_source_pclk();
        self.rcc.set_i2c1_source_pclk();

        self.i2c1.enable();

        self.rcc.enable_dac1();
        // Link DMA to USART1
        let usart1_channel_tx = self.dma1.request_channel();
        let usart1_channel_rx = self.dma1.request_channel();

        // Link DMA to I2C1
        let i2c1_channel_tx = self.dma1.request_channel();
        let i2c1_channel_rx = self.dma1.request_channel();

        if let (Some(tx), Some(rx)) = (usart1_channel_tx, usart1_channel_rx) {
            usart::Usart::set_dma(self.usart1, self.dma1, tx, rx);
        }

        if let (Some(tx), Some(rx)) = (i2c1_channel_tx, i2c1_channel_rx) {
            i2c::I2c::set_dma(&self.i2c1, self.dma1, tx, rx);
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
            I2C1_EV_IRQ => {
                self.i2c1.handle_interrupt();
                true
            }
            I2C1_ER_IRQ => {
                self.i2c1.handle_error();
                true
            }
            EXTI0_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line00);
                true
            }
            EXTI1_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line01);
                true
            }
            EXTI2_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line02);
                true
            }
            EXTI3_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line03);
                true
            }
            EXTI4_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line04);
                true
            }
            EXTI5_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line05);
                true
            }
            EXTI6_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line06);
                true
            }
            EXTI7_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line07);
                true
            }
            EXTI8_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line08);
                true
            }
            EXTI9_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line09);
                true
            }
            EXTI10_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line10);
                true
            }
            EXTI11_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line11);
                true
            }
            EXTI12_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line12);
                true
            }
            EXTI13_IRQ => {
                // EXTI13 (Button)
                self.exti.handle_interrupt(crate::exti::LineId::Line13);
                true
            }
            EXTI14_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line14);
                true
            }
            EXTI15_IRQ => {
                self.exti.handle_interrupt(crate::exti::LineId::Line15);
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

    fn init() {
        cortexm33::nvic::disable_all();
        cortexm33::nvic::clear_all_pending();
        cortexm33::nvic::enable_all();
    }

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
        cortexm33::nvic::has_pending()
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
