//! Chip trait setup.

use crate::console;
use crate::deferred_call_tasks::DeferredCallTask;

use core::fmt::Write;
use kernel::Chip;
use posix_x86_64;

use kernel::common::deferred_call;

pub struct Linux {
    mpu: (),
    userspace_kernel_boundary: posix_x86_64::syscall::SysCall,
    scheduler_timer: posix_x86_64::systick::SysTick,
}

impl Linux {
    pub unsafe fn new() -> Linux {
        Linux {
            mpu: (),
            userspace_kernel_boundary: posix_x86_64::syscall::SysCall::new(),
            scheduler_timer: posix_x86_64::systick::SysTick::new(),
        }
    }
}

impl Chip for Linux {
    type MPU = ();
    type UserspaceKernelBoundary = posix_x86_64::syscall::SysCall;
    type SchedulerTimer = posix_x86_64::systick::SysTick;
    type WatchDog = ();

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match task {
                        DeferredCallTask::Console => console::CONSOLE.handle_interrupt(),
                    }
                }
                //         if let Some(interrupt) = cortexm4::nvic::next_pending() {
                //             match interrupt {
                //                 nvic::DMA1_Stream1 => dma1::Dma1Peripheral::USART3_RX
                //                     .get_stream()
                //                     .handle_interrupt(),
                //                 nvic::DMA1_Stream2 => dma1::Dma1Peripheral::SPI3_RX
                //                     .get_stream()
                //                     .handle_interrupt(),
                //                 nvic::DMA1_Stream3 => dma1::Dma1Peripheral::USART3_TX
                //                     .get_stream()
                //                     .handle_interrupt(),
                //                 nvic::DMA1_Stream5 => dma1::Dma1Peripheral::USART2_RX
                //                     .get_stream()
                //                     .handle_interrupt(),
                //                 nvic::DMA1_Stream6 => dma1::Dma1Peripheral::USART2_TX
                //                     .get_stream()
                //                     .handle_interrupt(),
                //                 nvic::DMA1_Stream7 => dma1::Dma1Peripheral::SPI3_TX
                //                     .get_stream()
                //                     .handle_interrupt(),

                //                 nvic::USART2 => usart::USART2.handle_interrupt(),
                //                 nvic::USART3 => usart::USART3.handle_interrupt(),

                //                 nvic::ADC => adc::ADC1.handle_interrupt(),

                //                 nvic::I2C1_EV => i2c::I2C1.handle_event(),
                //                 nvic::I2C1_ER => i2c::I2C1.handle_error(),

                //                 nvic::SPI3 => spi::SPI3.handle_interrupt(),

                //                 nvic::EXTI0 => exti::EXTI.handle_interrupt(),
                //                 nvic::EXTI1 => exti::EXTI.handle_interrupt(),
                //                 nvic::EXTI2 => exti::EXTI.handle_interrupt(),
                //                 nvic::EXTI3 => exti::EXTI.handle_interrupt(),
                //                 nvic::EXTI4 => exti::EXTI.handle_interrupt(),
                //                 nvic::EXTI9_5 => exti::EXTI.handle_interrupt(),
                //                 nvic::EXTI15_10 => exti::EXTI.handle_interrupt(),

                //                 nvic::TIM2 => tim2::TIM2.handle_interrupt(),

                //                 _ => {
                //                     panic!("unhandled interrupt {}", interrupt);
                //                 }
                //             }

                //             let n = cortexm4::nvic::Nvic::new(interrupt);
                //             n.clear_pending();
                //             n.enable();
                // }
                else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { posix_x86_64::nvic::has_pending() || deferred_call::has_tasks() }
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        posix_x86_64::support::wfi();
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        posix_x86_64::support::atomic(f)
    }

    unsafe fn print_state(&self, write: &mut dyn Write) {
        posix_x86_64::print_cpu_state(write);
    }
}
