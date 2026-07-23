// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright OxidOS Automotive 2026.

use crate::{
    adc::{self, SamplingTime as AdcSamplingTime},
    aes::{self, ecb},
    crc, dac,
    dma::{ChannelId, Dma},
    exti, gpio, hash,
    i2c::{self, I2cSpeed},
    nvic::{
        ADC1_2_IRQ, AES_IRQ, EXTI0_IRQ, EXTI1_IRQ, EXTI2_IRQ, EXTI3_IRQ, EXTI4_IRQ, EXTI5_IRQ,
        EXTI6_IRQ, EXTI7_IRQ, EXTI8_IRQ, EXTI9_IRQ, EXTI10_IRQ, EXTI11_IRQ, EXTI12_IRQ, EXTI13_IRQ,
        EXTI14_IRQ, EXTI15_IRQ, GPDMA1_CH0_IRQ, GPDMA1_CH1_IRQ, GPDMA1_CH2_IRQ, GPDMA1_CH3_IRQ,
        GPDMA1_CH4_IRQ, GPDMA1_CH5_IRQ, GPDMA1_CH6_IRQ, GPDMA1_CH7_IRQ, GPDMA1_CH8_IRQ,
        GPDMA1_CH9_IRQ, GPDMA1_CH10_IRQ, GPDMA1_CH11_IRQ, GPDMA1_CH12_IRQ, GPDMA1_CH13_IRQ,
        GPDMA1_CH14_IRQ, GPDMA1_CH15_IRQ, HASH_IRQ, I2C1_ER_IRQ, I2C1_EV_IRQ, PKA_IRQ, SPI1_IRQ,
        TIM2_IRQ, USART1_IRQ,
    },
    pwr::{self, VoltageScale},
    rcc::{
        self,
        config::{ClockMuxConfig, RccConfig},
        values::{
            AHBPrescaler, APBPrescaler, Adcdacsel, I2csel, MsiRange, Rtcsel, Spi1sel, Sysclk,
            Usart1sel,
        },
    },
    rsa, rtc, spi, tim, usart,
};

use core::fmt::Write;
use kernel::deferred_call::DeferredCallClient;
use kernel::hil::spi::SpiMaster;
use kernel::hil::symmetric_encryption::AES256;
use kernel::platform::chip::Chip;
use kernel::platform::chip::InterruptService;

pub struct Stm32u5xx<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

pub struct Stm32u5xxDefaultPeripherals<'a> {
    pub rcc: rcc::Rcc,
    pub rtc: rtc::Rtc<'a>,
    pub tim2: tim::Tim2<'a>,
    pub tim3: tim::Pwm<'a>,
    pub usart1: usart::Usart<'a>,
    pub spi1: spi::Spi<'a>,
    pub i2c1: i2c::I2c<'a>,
    pub exti: &'a exti::Exti<'a>,
    pub dma1: &'a Dma,
    pub pwr: pwr::Pwr,
    pub adc1: adc::Adc<'a>,
    pub gpio_a: gpio::Port<'a>,
    pub gpio_b: gpio::Port<'a>,
    pub gpio_c: gpio::Port<'a>,
    pub pka: rsa::Pka<'a>,
    pub dac: dac::Dac,
    pub crc: crc::CRC<'a>,
    pub hash: hash::hash::Hash<'a>,
    pub aes: ecb::Aes<'a, AES256>,
}

impl<'a> Stm32u5xxDefaultPeripherals<'a> {
    pub fn new(exti: &'a exti::Exti<'a>, dma1: &'a Dma) -> Self {
        Self {
            rcc: rcc::Rcc::new(rcc::RCC_BASE),
            rtc: rtc::Rtc::new(rtc::RTC_BASE),
            tim2: tim::Tim2::new(tim::TIM2_BASE),
            tim3: tim::Pwm::new(tim::TIM3_BASE),
            usart1: usart::Usart::new(usart::USART1_BASE),
            spi1: spi::Spi::new(spi::SPI1_BASE),
            i2c1: i2c::I2c::new(i2c::I2C1_BASE),
            exti,
            dma1,
            pwr: pwr::Pwr::new(pwr::PWR_BASE),
            adc1: adc::Adc::new(adc::ADC1_BASE),
            gpio_a: gpio::Port::new(gpio::GPIO_A_BASE, exti, gpio::GpioPort::PortA),
            gpio_b: gpio::Port::new(gpio::GPIO_B_BASE, exti, gpio::GpioPort::PortB),
            gpio_c: gpio::Port::new(gpio::GPIO_C_BASE, exti, gpio::GpioPort::PortC),
            pka: rsa::Pka::new(rsa::PKA_BASE),
            dac: dac::Dac::new(dac::DAC_BASE),
            crc: crc::CRC::new(crc::CRC_BASE),
            hash: hash::hash::Hash::new(hash::regs::HASH_BASE),
            aes: aes::ecb::Aes::new(stm32u5xx_unsafe::aes::AesRegistersManager {
                registers: stm32u5xx_unsafe::aes::AES_BASE,
            }),
        }
    }

    pub fn init(&'static self) {
        // Enable clock routing to all used peripherals
        self.rcc.enable_tim2();
        self.rcc.enable_tim3();
        self.rcc.enable_dma1();
        self.rcc.enable_gpioa();
        self.rcc.enable_gpiob();
        self.rcc.enable_gpioc();
        self.rcc.enable_usart1();
        self.rcc.enable_aes();
        self.rcc.enable_syscfg();
        self.rcc.enable_pwr();
        self.rcc.enable_adc1();
        self.rcc.enable_dac1();
        self.rcc.enable_hash();
        self.rcc.enable_trng();
        self.rcc.enable_crc();
        self.rcc.enable_pka();
        self.rcc.enable_spi1();
        self.rcc.enable_i2c1();
        self.rcc.enable_rtc_apb();

        // Select which clocks to enable, and how to configure them
        let mut rcc_config = RccConfig {
            msis: Some(MsiRange::Range4mhz),
            msik: Some(MsiRange::Range4mhz),
            hsi: true, // 16MHz oscillator enabled (for SYSCLK/ADC/DAC)
            hse: None,
            hsi48: false,
            lsi: true, // 32kHz oscillator enabled (for RTC)
            pll1: None,
            pll2: None,
            pll3: None,
            sys: Sysclk::Hsi, // 16MHz system clock
            ahb_pre: AHBPrescaler::Div1,
            apb1_pre: APBPrescaler::Div1,
            apb2_pre: APBPrescaler::Div1,
            apb3_pre: APBPrescaler::Div1,
            voltage_range: VoltageScale::Range1, // allow highest frequencies
            mux: ClockMuxConfig::default(),
        };

        // Use HSI (16MHz) for SYSCLK, ADC and DAC
        rcc_config.mux.adcdacsel = Adcdacsel::Hsi;
        // Use PCLK2 for USART1 (it's the default anyways)
        rcc_config.mux.usart1sel = Usart1sel::Pclk2;
        // Use PCLK1 for I2C1 (it's the default anyways)
        rcc_config.mux.i2c1sel = I2csel::Pclk1;
        // Use PCLK2 for SPI1 (it's the default anyways)
        rcc_config.mux.spi1sel = Spi1sel::Pclk2;
        // Use LSI for RTC
        rcc_config.mux.rtcsel = Rtcsel::Lsi;

        // Backup domain write protection needs to be disabled to be able to change the RCC_BDCR register
        // This is necessary for enabling the LSI oscillator and configuring the RTC
        self.pwr.disable_backup_domain_write_protection();

        // Now the RTC clock can be enabled
        self.rcc.enable_rtc();

        // Initialize the RCC
        // This returns a structure containing the effective calculated frequency for all clocks in the clock tree
        let clocks = self.rcc.init(rcc_config, &self.pwr);

        // Provide a copy of that structure to each peripheral that needs it
        self.usart1.set_clocks(clocks);
        self.tim2.set_clocks(clocks);
        self.tim3.set_clocks(clocks);
        self.spi1.set_clocks(clocks);
        self.i2c1.set_clocks(clocks);
        self.rtc.set_clocks(clocks);

        // Activate the independent analog supply, needed for analog peripherals
        self.pwr.validate_vdda();

        // Register deferred call clients
        self.usart1.register();
        self.hash.register();
        self.crc.register();
        self.rtc.register();

        // Link DMA to USART1
        let usart1_channel_tx = self.dma1.request_channel();
        let usart1_channel_rx = self.dma1.request_channel();
        if let (Some(tx), Some(rx)) = (usart1_channel_tx, usart1_channel_rx) {
            usart::Usart::set_dma(&self.usart1, self.dma1, tx, rx);
        }

        // Link DMA to HASH
        let hash_channel = self.dma1.request_channel();
        if let Some(tx) = hash_channel {
            hash::hash::Hash::set_dma(&self.hash, self.dma1, tx);
        }

        // Link DMA to AES
        let aes_in_channel = self.dma1.request_channel();
        let aes_out_channel = self.dma1.request_channel();
        if let (Some(in_channel), Some(out_channel)) = (aes_in_channel, aes_out_channel) {
            aes::ecb::Aes::set_dma(&self.aes, self.dma1, in_channel, out_channel);
        }

        // Link DMA to SPI1
        let spi1_channel_tx = self.dma1.request_channel();
        let spi1_channel_rx = self.dma1.request_channel();
        if let (Some(tx), Some(rx)) = (spi1_channel_tx, spi1_channel_rx) {
            spi::Spi::set_dma(&self.spi1, self.dma1, tx, rx);
        }

        // Link DMA to I2C1
        let i2c1_channel_tx = self.dma1.request_channel();
        let i2c1_channel_rx = self.dma1.request_channel();
        if let (Some(tx), Some(rx)) = (i2c1_channel_tx, i2c1_channel_rx) {
            i2c::I2c::set_dma(&self.i2c1, self.dma1, tx, rx);
        }

        // Enable ADC
        // As explained in the driver, an application can't change the ADC sampling time, so it's hardcoded here
        self.adc1.enable(AdcSamplingTime::ClockCycles20);

        // Set up the RTC mode (configure prescalers, 24h format, default date/time)
        let _ = self.rtc.init_mode();

        // Initialize SPI1
        let _ = self.spi1.init();

        // Enable I2C1 and configure it at 100kHz
        self.i2c1.enable();
        self.i2c1.set_speed(I2cSpeed::Speed100k);
    }
}

impl InterruptService for Stm32u5xxDefaultPeripherals<'_> {
    fn service_interrupt(&self, interrupt: u32) -> bool {
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
            SPI1_IRQ => {
                // SPI1
                self.spi1.handle_interrupt();
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
            PKA_IRQ => {
                self.pka.handle_interrupt();
                true
            }
            HASH_IRQ => {
                self.hash.handle_interupts();
                true
            }
            AES_IRQ => {
                self.aes.handle_interrupt();
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
        while let Some(interrupt) = cortexm33::nvic::next_pending() {
            if !self.interrupt_service.service_interrupt(interrupt) {
                panic!("unhandled interrupt {}", interrupt);
            }

            let n = cortexm33::nvic::Nvic::new(interrupt);
            n.clear_pending();
            n.enable();
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

    fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm33::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, write: &mut dyn Write) {
        let _ = write.write_str("Cortex-M33 state\n");
    }
}
