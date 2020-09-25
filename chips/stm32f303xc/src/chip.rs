//! Chip trait setup.

use core::fmt::Write;
use cortexm4;
use kernel::common::deferred_call;
use kernel::Chip;

use crate::adc;
use crate::deferred_call_tasks::DeferredCallTask;
use crate::exti;
use crate::flash;
use crate::i2c;
use crate::nvic;
use crate::spi;
use crate::tim2;
use crate::usart;

pub struct Stm32f3xx {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    scheduler_timer: cortexm4::systick::SysTick,
}

impl Stm32f3xx {
    pub unsafe fn new() -> Stm32f3xx {
        Stm32f3xx {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            scheduler_timer: cortexm4::systick::SysTick::new(),
        }
    }
}

impl Chip for Stm32f3xx {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SchedulerTimer = cortexm4::systick::SysTick;
    type WatchDog = ();

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match task {
                        DeferredCallTask::Flash => flash::FLASH.handle_interrupt(),
                    }
                } else if let Some(interrupt) = cortexm4::nvic::next_pending() {
                    match interrupt {
                        nvic::USART1 => usart::USART1.handle_interrupt(),

                        nvic::TIM2 => tim2::TIM2.handle_interrupt(),

                        nvic::SPI1 => spi::SPI1.handle_interrupt(),

                        nvic::FLASH => flash::FLASH.handle_interrupt(),

                        nvic::I2C1_EV => i2c::I2C1.handle_event(),
                        nvic::I2C1_ER => i2c::I2C1.handle_error(),
                        nvic::ADC1_2 => adc::ADC1.handle_interrupt(),

                        nvic::EXTI0 => exti::EXTI.handle_interrupt(),
                        nvic::EXTI1 => exti::EXTI.handle_interrupt(),
                        nvic::EXTI2 => exti::EXTI.handle_interrupt(),
                        nvic::EXTI3 => exti::EXTI.handle_interrupt(),
                        nvic::EXTI4 => exti::EXTI.handle_interrupt(),
                        nvic::EXTI9_5 => exti::EXTI.handle_interrupt(),
                        nvic::EXTI15_10 => exti::EXTI.handle_interrupt(),

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
