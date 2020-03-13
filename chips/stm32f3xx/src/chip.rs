//! Chip trait setup.

use core::fmt::Write;
use cortexm4;
use kernel::common::deferred_call;
use kernel::Chip;

use crate::deferred_call_tasks::Task;
use crate::nvic;
use crate::tim2;
use crate::usart;

pub struct Stm32f3xx {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    systick: cortexm4::systick::SysTick,
}

impl Stm32f3xx {
    pub unsafe fn new() -> Stm32f3xx {
        Stm32f3xx {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            systick: cortexm4::systick::SysTick::new(),
        }
    }
}

impl Chip for Stm32f3xx {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SysTick = cortexm4::systick::SysTick;

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match task {
                        Task::Nop => {}
                    }
                } else if let Some(interrupt) = cortexm4::nvic::next_pending() {
                    match interrupt {
                        nvic::USART1 => usart::USART1.handle_interrupt(),

                        nvic::TIM2 => tim2::TIM2.handle_interrupt(),
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
