//! Interrupt mapping and DMA channel setup.

use crate::acifc;
use crate::adc;
use crate::aes;
use crate::ast;
use crate::crccu;
use crate::dac;
use crate::deferred_call_tasks::Task;
use crate::dma;
use crate::eic;
use crate::flashcalw;
use crate::gpio;
use crate::i2c;
use crate::nvic;
use crate::pm;
use crate::spi;
use crate::trng;
use crate::usart;
use crate::usbc;

use cortexm4;
use kernel::common::deferred_call;
use kernel::Chip;

pub struct Sam4l {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    systick: cortexm4::systick::SysTick,
}

impl Sam4l {
    pub unsafe fn new() -> Sam4l {
        usart::USART0.set_dma(&mut dma::DMA_CHANNELS[0], &mut dma::DMA_CHANNELS[1]);
        dma::DMA_CHANNELS[0].initialize(&mut usart::USART0, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[1].initialize(&mut usart::USART0, dma::DMAWidth::Width8Bit);

        usart::USART1.set_dma(&mut dma::DMA_CHANNELS[2], &mut dma::DMA_CHANNELS[3]);
        dma::DMA_CHANNELS[2].initialize(&mut usart::USART1, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[3].initialize(&mut usart::USART1, dma::DMAWidth::Width8Bit);

        usart::USART2.set_dma(&mut dma::DMA_CHANNELS[4], &mut dma::DMA_CHANNELS[5]);
        dma::DMA_CHANNELS[4].initialize(&mut usart::USART2, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[5].initialize(&mut usart::USART2, dma::DMAWidth::Width8Bit);

        usart::USART3.set_dma(&mut dma::DMA_CHANNELS[6], &mut dma::DMA_CHANNELS[7]);
        dma::DMA_CHANNELS[6].initialize(&mut usart::USART3, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[7].initialize(&mut usart::USART3, dma::DMAWidth::Width8Bit);

        spi::SPI.set_dma(&mut dma::DMA_CHANNELS[8], &mut dma::DMA_CHANNELS[9]);
        dma::DMA_CHANNELS[8].initialize(&mut spi::SPI, dma::DMAWidth::Width8Bit);
        dma::DMA_CHANNELS[9].initialize(&mut spi::SPI, dma::DMAWidth::Width8Bit);

        i2c::I2C0.set_dma(&dma::DMA_CHANNELS[10]);
        dma::DMA_CHANNELS[10].initialize(&mut i2c::I2C0, dma::DMAWidth::Width8Bit);

        i2c::I2C1.set_dma(&dma::DMA_CHANNELS[11]);
        dma::DMA_CHANNELS[11].initialize(&mut i2c::I2C1, dma::DMAWidth::Width8Bit);

        i2c::I2C2.set_dma(&dma::DMA_CHANNELS[12]);
        dma::DMA_CHANNELS[12].initialize(&mut i2c::I2C2, dma::DMAWidth::Width8Bit);

        adc::ADC0.set_dma(&dma::DMA_CHANNELS[13]);
        dma::DMA_CHANNELS[13].initialize(&mut adc::ADC0, dma::DMAWidth::Width16Bit);

        Sam4l {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            systick: cortexm4::systick::SysTick::new(),
        }
    }
}

impl Chip for Sam4l {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SysTick = cortexm4::systick::SysTick;

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match task {
                        Task::Flashcalw => flashcalw::FLASH_CONTROLLER.handle_interrupt(),
                    }
                } else if let Some(interrupt) = cortexm4::nvic::next_pending() {
                    match interrupt {
                        nvic::ASTALARM => ast::AST.handle_interrupt(),

                        nvic::USART0 => usart::USART0.handle_interrupt(),
                        nvic::USART1 => usart::USART1.handle_interrupt(),
                        nvic::USART2 => usart::USART2.handle_interrupt(),
                        nvic::USART3 => usart::USART3.handle_interrupt(),

                        nvic::PDCA0 => dma::DMA_CHANNELS[0].handle_interrupt(),
                        nvic::PDCA1 => dma::DMA_CHANNELS[1].handle_interrupt(),
                        nvic::PDCA2 => dma::DMA_CHANNELS[2].handle_interrupt(),
                        nvic::PDCA3 => dma::DMA_CHANNELS[3].handle_interrupt(),
                        nvic::PDCA4 => dma::DMA_CHANNELS[4].handle_interrupt(),
                        nvic::PDCA5 => dma::DMA_CHANNELS[5].handle_interrupt(),
                        nvic::PDCA6 => dma::DMA_CHANNELS[6].handle_interrupt(),
                        nvic::PDCA7 => dma::DMA_CHANNELS[7].handle_interrupt(),
                        nvic::PDCA8 => dma::DMA_CHANNELS[8].handle_interrupt(),
                        nvic::PDCA9 => dma::DMA_CHANNELS[9].handle_interrupt(),
                        nvic::PDCA10 => dma::DMA_CHANNELS[10].handle_interrupt(),
                        nvic::PDCA11 => dma::DMA_CHANNELS[11].handle_interrupt(),
                        nvic::PDCA12 => dma::DMA_CHANNELS[12].handle_interrupt(),
                        nvic::PDCA13 => dma::DMA_CHANNELS[13].handle_interrupt(),
                        nvic::PDCA14 => dma::DMA_CHANNELS[14].handle_interrupt(),
                        nvic::PDCA15 => dma::DMA_CHANNELS[15].handle_interrupt(),

                        nvic::CRCCU => crccu::CRCCU.handle_interrupt(),
                        nvic::USBC => usbc::USBC.handle_interrupt(),

                        nvic::GPIO0 => gpio::PA.handle_interrupt(),
                        nvic::GPIO1 => gpio::PA.handle_interrupt(),
                        nvic::GPIO2 => gpio::PA.handle_interrupt(),
                        nvic::GPIO3 => gpio::PA.handle_interrupt(),
                        nvic::GPIO4 => gpio::PB.handle_interrupt(),
                        nvic::GPIO5 => gpio::PB.handle_interrupt(),
                        nvic::GPIO6 => gpio::PB.handle_interrupt(),
                        nvic::GPIO7 => gpio::PB.handle_interrupt(),
                        nvic::GPIO8 => gpio::PC.handle_interrupt(),
                        nvic::GPIO9 => gpio::PC.handle_interrupt(),
                        nvic::GPIO10 => gpio::PC.handle_interrupt(),
                        nvic::GPIO11 => gpio::PC.handle_interrupt(),

                        nvic::SPI => spi::SPI.handle_interrupt(),

                        nvic::TWIM0 => i2c::I2C0.handle_interrupt(),
                        nvic::TWIM1 => i2c::I2C1.handle_interrupt(),
                        nvic::TWIM2 => i2c::I2C2.handle_interrupt(),
                        nvic::TWIM3 => i2c::I2C3.handle_interrupt(),

                        nvic::TWIS0 => i2c::I2C0.handle_slave_interrupt(),
                        nvic::TWIS1 => i2c::I2C1.handle_slave_interrupt(),

                        nvic::HFLASHC => flashcalw::FLASH_CONTROLLER.handle_interrupt(),
                        nvic::ADCIFE => adc::ADC0.handle_interrupt(),
                        nvic::DACC => dac::DAC.handle_interrupt(),
                        nvic::ACIFC => acifc::ACIFC.handle_interrupt(),

                        nvic::TRNG => trng::TRNG.handle_interrupt(),
                        nvic::AESA => aes::AES.handle_interrupt(),

                        nvic::EIC1 => eic::EIC.handle_interrupt(eic::Line::Ext1),
                        nvic::EIC2 => eic::EIC.handle_interrupt(eic::Line::Ext2),
                        nvic::EIC3 => eic::EIC.handle_interrupt(eic::Line::Ext3),
                        nvic::EIC4 => eic::EIC.handle_interrupt(eic::Line::Ext4),
                        nvic::EIC5 => eic::EIC.handle_interrupt(eic::Line::Ext5),
                        nvic::EIC6 => eic::EIC.handle_interrupt(eic::Line::Ext6),
                        nvic::EIC7 => eic::EIC.handle_interrupt(eic::Line::Ext7),
                        nvic::EIC8 => eic::EIC.handle_interrupt(eic::Line::Ext8),

                        _ => {
                            panic!("unhandled interrupt {}", interrupt);
                        }
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
        unsafe { cortexm4::nvic::has_pending() || deferred_call::has_tasks() }
    }

    fn mpu(&self) -> &cortexm4::mpu::MPU {
        &self.mpu
    }

    fn systick(&self) -> &cortexm4::systick::SysTick {
        &self.systick
    }

    fn userspace_kernel_boundary(&self) -> &cortexm4::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        if pm::deep_sleep_ready() {
            unsafe {
                cortexm4::scb::set_sleepdeep();
            }
        } else {
            unsafe {
                cortexm4::scb::unset_sleepdeep();
            }
        }

        unsafe {
            cortexm4::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4::support::atomic(f)
    }
}
