//! Chip trait setup.

use core::fmt::Write;
use cortexm4;
use kernel::common::deferred_call;
use kernel::Chip;
use kernel::InterruptService;

use crate::deferred_call_tasks::DeferredCallTask;
use crate::nvic;
use crate::wdt;

pub struct Stm32f3xx<'a, I: InterruptService<DeferredCallTask> + 'a> {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    scheduler_timer: cortexm4::systick::SysTick,
    watchdog: wdt::WindoWdg<'a>,
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
        }
    }

    pub fn setup_circular_deps(&'a self) {
        self.gpio_ports.setup_circular_deps();
    }
}

impl<'a> InterruptService<DeferredCallTask> for Stm32f3xxDefaultPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            nvic::USART1 => self.usart1.handle_interrupt(),

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

    unsafe fn service_deferred_call(&self, task: DeferredCallTask) -> bool {
        match task {
            DeferredCallTask::Flash => self.flash.handle_interrupt(),
        }
        true
    }
}

impl<'a, I: InterruptService<DeferredCallTask> + 'a> Stm32f3xx<'a, I> {
    pub unsafe fn new(interrupt_service: &'a I, rcc: &'a crate::rcc::Rcc) -> Self {
        Self {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            scheduler_timer: cortexm4::systick::SysTick::new(),
            watchdog: wdt::WindoWdg::new(rcc),
            interrupt_service,
        }
    }

    pub fn enable_watchdog(&self) {
        self.watchdog.enable();
    }
}

impl<'a, I: InterruptService<DeferredCallTask> + 'a> Chip for Stm32f3xx<'a, I> {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = wdt::WindoWdg<'a>;

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    if !self.interrupt_service.service_deferred_call(task) {
                        panic!("unhandled deferred call");
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
        &self.watchdog
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
