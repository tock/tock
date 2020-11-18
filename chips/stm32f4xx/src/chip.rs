//! Chip trait setup.

use core::fmt::Write;
use cortexm4;
use kernel::Chip;

use kernel::common::deferred_call;
use kernel::InterruptService;

use crate::dma1;
use crate::nvic;

use crate::deferred_calls::DeferredCallTask;

pub struct Stm32f4xx<'a, I: InterruptService<DeferredCallTask> + 'a> {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    scheduler_timer: cortexm4::systick::SysTick,
    interrupt_service: &'a I,
}

pub struct Stm32f4xxDefaultPeripherals<'a> {
    pub adc1: crate::adc::Adc<'a>,
    pub dma_streams: [crate::dma1::Stream<'a>; 8],
    pub exti: &'a crate::exti::Exti<'a>,
    pub i2c1: crate::i2c::I2C<'a>,
    pub spi3: crate::spi::Spi<'a>,
    pub tim2: crate::tim2::Tim2<'a>,
    pub usart2: crate::usart::Usart<'a>,
    pub usart3: crate::usart::Usart<'a>,
    pub gpio_ports: crate::gpio::GpioPorts<'a>,
    pub fsmc: crate::fsmc::Fsmc<'a>,
}

impl<'a> Stm32f4xxDefaultPeripherals<'a> {
    pub fn new(
        rcc: &'a crate::rcc::Rcc,
        exti: &'a crate::exti::Exti<'a>,
        dma: &'a crate::dma1::Dma1<'a>,
    ) -> Self {
        Self {
            adc1: crate::adc::Adc::new(rcc),
            dma_streams: crate::dma1::new_dma1_stream(dma),
            exti,
            i2c1: crate::i2c::I2C::new(rcc),
            spi3: crate::spi::Spi::new(
                crate::spi::SPI3_BASE,
                crate::spi::SpiClock(crate::rcc::PeripheralClock::new(
                    crate::rcc::PeripheralClockType::APB1(crate::rcc::PCLK1::SPI3),
                    rcc,
                )),
                crate::dma1::Dma1Peripheral::SPI3_TX,
                crate::dma1::Dma1Peripheral::SPI3_RX,
            ),
            tim2: crate::tim2::Tim2::new(rcc),
            usart2: crate::usart::Usart::new_usart2(rcc),
            usart3: crate::usart::Usart::new_usart3(rcc),
            gpio_ports: crate::gpio::GpioPorts::new(rcc, exti),
            fsmc: crate::fsmc::Fsmc::new(
                [
                    Some(crate::fsmc::FSMC_BANK1),
                    None,
                    Some(crate::fsmc::FSMC_BANK3),
                    None,
                ],
                rcc,
            ),
        }
    }

    pub fn setup_circular_deps(&'a self) {
        self.gpio_ports.setup_circular_deps();
    }
}

impl<'a> InterruptService<DeferredCallTask> for Stm32f4xxDefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            nvic::DMA1_Stream1 => self.dma_streams
                [dma1::Dma1Peripheral::USART3_RX.get_stream_idx()]
            .handle_interrupt(),
            nvic::DMA1_Stream2 => {
                self.dma_streams[dma1::Dma1Peripheral::SPI3_RX.get_stream_idx()].handle_interrupt()
            }
            nvic::DMA1_Stream3 => self.dma_streams
                [dma1::Dma1Peripheral::USART3_TX.get_stream_idx()]
            .handle_interrupt(),
            nvic::DMA1_Stream5 => self.dma_streams
                [dma1::Dma1Peripheral::USART2_RX.get_stream_idx()]
            .handle_interrupt(),
            nvic::DMA1_Stream6 => self.dma_streams
                [dma1::Dma1Peripheral::USART2_TX.get_stream_idx()]
            .handle_interrupt(),
            nvic::DMA1_Stream7 => {
                self.dma_streams[dma1::Dma1Peripheral::SPI3_TX.get_stream_idx()].handle_interrupt()
            }

            nvic::USART2 => self.usart2.handle_interrupt(),
            nvic::USART3 => self.usart3.handle_interrupt(),

            nvic::ADC => self.adc1.handle_interrupt(),

            nvic::I2C1_EV => self.i2c1.handle_event(),
            nvic::I2C1_ER => self.i2c1.handle_error(),

            nvic::SPI3 => self.spi3.handle_interrupt(),

            nvic::EXTI0 => self.exti.handle_interrupt(),
            nvic::EXTI1 => self.exti.handle_interrupt(),
            nvic::EXTI2 => self.exti.handle_interrupt(),
            nvic::EXTI3 => self.exti.handle_interrupt(),
            nvic::EXTI4 => self.exti.handle_interrupt(),
            nvic::EXTI9_5 => self.exti.handle_interrupt(),
            nvic::EXTI15_10 => self.exti.handle_interrupt(),

            nvic::TIM2 => self.tim2.handle_interrupt(),

            _ => return false,
        }
        true
    }

    unsafe fn service_deferred_call(&self, task: DeferredCallTask) -> bool {
        match task {
            DeferredCallTask::Fsmc => self.fsmc.handle_interrupt(),
        }
        true
    }
}

impl<'a, I: InterruptService<DeferredCallTask> + 'a> Stm32f4xx<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I) -> Self {
        Self {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            scheduler_timer: cortexm4::systick::SysTick::new(),
            interrupt_service,
        }
    }
}

impl<'a, I: InterruptService<DeferredCallTask> + 'a> Chip for Stm32f4xx<'a, I> {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = ();

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    if !self.interrupt_service.service_deferred_call(task) {
                        panic!("Unhandled deferred call");
                    }
                } else if let Some(interrupt) = cortexm4::nvic::next_pending() {
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
        unsafe { cortexm4::nvic::has_pending() || deferred_call::has_tasks() }
    }

    fn mpu(&self) -> &cortexm4::mpu::MPU {
        &self.mpu
    }

    fn scheduler_timer(&self) -> &cortexm4::systick::SysTick {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &cortexm4::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm4::scb::unset_sleepdeep();
            cortexm4::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm4::support::atomic(f)
    }

    unsafe fn print_state(&self, write: &mut dyn Write) {
        cortexm4::print_cortexm4_state(write);
    }
}
