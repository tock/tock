//! Chip trait setup.

use core::fmt::Write;
use cortexm7;
use kernel::Chip;

use crate::gpt1;
use crate::lpi2c;
use crate::lpuart;
use crate::nvic;

pub struct Imxrt1050 {
    mpu: cortexm7::mpu::MPU,
    userspace_kernel_boundary: cortexm7::syscall::SysCall,
    scheduler_timer: cortexm7::systick::SysTick,
}

impl Imxrt1050 {
    pub unsafe fn new() -> Imxrt1050 {
        Imxrt1050 {
            mpu: cortexm7::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm7::syscall::SysCall::new(),
            scheduler_timer: cortexm7::systick::SysTick::new_with_calibration(792_000_000),
        }
    }
}

impl Chip for Imxrt1050 {
    type MPU = cortexm7::mpu::MPU;
    type UserspaceKernelBoundary = cortexm7::syscall::SysCall;
    type SchedulerTimer = cortexm7::systick::SysTick;
    type WatchDog = ();

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(interrupt) = cortexm7::nvic::next_pending() {
                    match interrupt {
                        nvic::LPUART1 => lpuart::LPUART1.handle_interrupt(),
                        nvic::LPI2C1 => lpi2c::LPI2C1.handle_event(),
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
        unsafe { cortexm7::nvic::has_pending() }
    }

    fn mpu(&self) -> &cortexm7::mpu::MPU {
        &self.mpu
    }

    fn scheduler_timer(&self) -> &cortexm7::systick::SysTick {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
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
