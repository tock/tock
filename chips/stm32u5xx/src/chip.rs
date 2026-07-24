// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright OxidOS Automotive 2026.

use crate::adc::{self, SamplingTime as AdcSamplingTime};
use crate::dma::{ChannelId, Dma};
use crate::gpio;
use crate::nvic::{
    ADC1_2_IRQ, EXTI0_IRQ, EXTI1_IRQ, EXTI2_IRQ, EXTI3_IRQ, EXTI4_IRQ, EXTI5_IRQ, EXTI6_IRQ,
    EXTI7_IRQ, EXTI8_IRQ, EXTI9_IRQ, EXTI10_IRQ, EXTI11_IRQ, EXTI12_IRQ, EXTI13_IRQ, EXTI14_IRQ,
    EXTI15_IRQ, GPDMA1_CH0_IRQ, GPDMA1_CH1_IRQ, GPDMA1_CH2_IRQ, GPDMA1_CH3_IRQ, GPDMA1_CH4_IRQ,
    GPDMA1_CH5_IRQ, GPDMA1_CH6_IRQ, GPDMA1_CH7_IRQ, GPDMA1_CH8_IRQ, GPDMA1_CH9_IRQ,
    GPDMA1_CH10_IRQ, GPDMA1_CH11_IRQ, GPDMA1_CH12_IRQ, GPDMA1_CH13_IRQ, GPDMA1_CH14_IRQ,
    GPDMA1_CH15_IRQ, TIM2_IRQ, USART1_IRQ,
};
use crate::pwr;
use crate::rcc;
use crate::rtc;
use crate::tim;
use crate::usart;
use crate::{dac, exti};

use core::fmt::Write;
use kernel::debug;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

pub struct Stm32u5xx<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

pub struct Stm32u5xxDefaultPeripherals<'a> {
    pub rcc: &'a rcc::Rcc,
    pub rtc: rtc::Rtc<'a>,
    pub tim2: tim::Tim2<'a>,
    pub tim3: tim::Pwm<'a>,
    pub usart1: &'a usart::Usart<'a>,
    pub exti: &'a exti::Exti<'a>,
    pub dma1: &'a Dma,
    pub pwr: pwr::Pwr,
    pub adc1: adc::Adc<'a>,
    pub gpio_a: gpio::Port<'a>,
    pub gpio_b: gpio::Port<'a>,
    pub gpio_c: gpio::Port<'a>,
    pub dac: dac::Dac,
}

fn enable_tim2_clock() {
    let rcc = rcc::Rcc::new(rcc::RCC_BASE);
    rcc.enable_tim2();
}
fn enable_tim3_clock() {
    let rcc = rcc::Rcc::new(rcc::RCC_BASE);
    rcc.enable_tim3();
}

fn enable_dac1_clock() {
    let rcc = rcc::Rcc::new(rcc::RCC_BASE);
    rcc.enable_dac1();
}

impl<'a> Stm32u5xxDefaultPeripherals<'a> {
    pub fn new(
        usart1: &'a usart::Usart<'a>,
        exti: &'a exti::Exti<'a>,
        dma1: &'a Dma,
        rcc: &'a rcc::Rcc,
    ) -> Self {
        Self {
            rcc,
            rtc: rtc::Rtc::new(rcc),
            tim2: tim::Tim2::new(tim::TIM2_BASE, enable_tim2_clock),
            tim3: tim::Pwm::new(
                tim::TIM3_BASE,
                enable_tim3_clock,
                tim::ClockSource::RESET_DEFAULT,
            ),

            usart1,
            exti,
            dma1,
            pwr: pwr::Pwr::new(),
            adc1: adc::Adc::new(),
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
        self.rcc.enable_gpioc();
        self.rcc.enable_usart1();
        self.rcc.enable_syscfg();
        self.rcc.enable_pwr();
        self.rcc.enable_adc1();
        self.rcc.set_usart1_source_pclk();

        // ADC
        // Decided to use clock source HSI16, so that needs to be enabled in the RCC too
        self.rcc.set_adcdacsel_source_hsi16();
        self.rcc.enable_hsi16();
        // For the ADC's voltage regulator to receive power, V_DDA must be validated (SVMCR.ASV) in PWR
        self.pwr.validate_vdda();
        // As explained in the driver, an application can't change the samplling time, so it's hardcoded here
        self.adc1.enable(AdcSamplingTime::ClockCycles20);

        self.rcc.enable_dac1();
        // Link DMA to USART1
        let usart1_channel_tx = self.dma1.request_channel();
        let usart1_channel_rx = self.dma1.request_channel();
        if let (Some(tx), Some(rx)) = (usart1_channel_tx, usart1_channel_rx) {
            usart::Usart::set_dma(self.usart1, self.dma1, tx, rx);
        }

        // Turn on the RTC clock and unlock the backup domain.
        // We handle errors here such that a failure doesn't halt the kernel.
        if let Err(e) = self.rtc.initialize_clock() {
            debug!("{:?}", e)
        }
        // Set up the RTC mode. (configure prescalers, 24h format, default date/time)
        // This requires the previous step, the clock and bckup domain to have been sucessfully initialized.
        if let Err(e) = self.rtc.init_mode() {
            debug!("{:?}", e)
        }
    }
}

impl InterruptService for Stm32u5xxDefaultPeripherals<'_> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            ADC1_2_IRQ => {
                // ADC1
                self.adc1.handle_interrupt();
                true
            }
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
