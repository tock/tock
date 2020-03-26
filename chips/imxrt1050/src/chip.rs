//! Chip trait setup.

use core::fmt::Write;
use cortexm7;
use kernel::common::deferred_call;
use kernel::Chip;

use crate::deferred_call_tasks::Task;
// use crate::dma1;
// use crate::exti;
use crate::nvic;
// use crate::spi;
use crate::gpt1;
// use crate::usart;

pub struct Imxrt1050 {
    mpu: cortexm7::mpu::MPU,
    userspace_kernel_boundary: cortexm7::syscall::SysCall,
    systick: cortexm7::systick::SysTick,
}

impl Imxrt1050 {
    pub unsafe fn new() -> Imxrt1050 {
        Imxrt1050 {
            mpu: cortexm7::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm7::syscall::SysCall::new(),
            systick: cortexm7::systick::SysTick::new(),
        }
    }
}

impl Chip for Imxrt1050 {
    type MPU = cortexm7::mpu::MPU;
    type UserspaceKernelBoundary = cortexm7::syscall::SysCall;
    type SysTick = cortexm7::systick::SysTick;

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match task {
                        Task::Nop => {}
                    }
                } else if let Some(interrupt) = cortexm7::nvic::next_pending() {
                    match interrupt {
                        // nvic::DMA1_Stream1 => dma1::Dma1Peripheral::USART3_RX
                        //     .get_stream()
                        //     .handle_interrupt(),
                        // nvic::DMA1_Stream2 => dma1::Dma1Peripheral::SPI3_RX
                        //     .get_stream()
                        //     .handle_interrupt(),
                        // nvic::DMA1_Stream3 => dma1::Dma1Peripheral::USART3_TX
                        //     .get_stream()
                        //     .handle_interrupt(),
                        // nvic::DMA1_Stream5 => dma1::Dma1Peripheral::USART2_RX
                        //     .get_stream()
                        //     .handle_interrupt(),
                        // nvic::DMA1_Stream6 => dma1::Dma1Peripheral::USART2_TX
                        //     .get_stream()
                        //     .handle_interrupt(),
                        // nvic::DMA1_Stream7 => dma1::Dma1Peripheral::SPI3_TX
                        //     .get_stream()
                        //     .handle_interrupt(),

                        // nvic::USART2 => usart::USART2.handle_interrupt(),
                        // nvic::USART3 => usart::USART3.handle_interrupt(),

                        // nvic::SPI3 => spi::SPI3.handle_interrupt(),

                        // nvic::EXTI0 => exti::EXTI.handle_interrupt(),
                        // nvic::EXTI1 => exti::EXTI.handle_interrupt(),
                        // nvic::EXTI2 => exti::EXTI.handle_interrupt(),
                        // nvic::EXTI3 => exti::EXTI.handle_interrupt(),
                        // nvic::EXTI4 => exti::EXTI.handle_interrupt(),
                        // nvic::EXTI9_5 => exti::EXTI.handle_interrupt(),
                        // nvic::EXTI15_10 => exti::EXTI.handle_interrupt(),

                        // nvic::TIM2 => tim2::TIM2.handle_interrupt(),
                        nvic::GPT1 => gpt1::GPT1.handle_interrupt(),

                        _ => {
                            panic!("unhandled interrupt {}", interrupt);
                        }
                    }

                    let n = cortexm7::nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm7::nvic::has_pending() || deferred_call::has_tasks() }
        // false
    }

    fn mpu(&self) -> &cortexm7::mpu::MPU {
        &self.mpu
    }

    fn systick(&self) -> &cortexm7::systick::SysTick {
        &self.systick
    }

    fn userspace_kernel_boundary(&self) -> &cortexm7::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        unsafe {
            cortexm7::scb::unset_sleepdeep();
            cortexm7::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm7::support::atomic(f)
    }

    unsafe fn print_state(&self, write: &mut dyn Write) {
        cortexm7::print_cortexm7_state(write);
    }
}
