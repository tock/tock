use crate::deferred_call_tasks::DeferredCallTask;
use crate::interrupt_service::InterruptService;
use crate::nvmc;
use core::fmt::Write;
use cortexm4::{self, nvic};
use kernel::common::deferred_call;
use kernel::debug;

pub struct NRF52<I: InterruptService> {
    mpu: cortexm4::mpu::MPU,
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    systick: cortexm4::systick::SysTick,
    interrupt_service: I,
}

impl<I: InterruptService> NRF52<I> {
    pub unsafe fn new(interrupt_service: I) -> NRF52<I> {
        NRF52 {
            mpu: cortexm4::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            // The NRF52's systick is uncalibrated, but is clocked from the
            // 64Mhz CPU clock.
            systick: cortexm4::systick::SysTick::new_with_calibration(64000000),
            interrupt_service,
        }
    }
}

impl<I: InterruptService> kernel::Chip for NRF52<I> {
    type MPU = cortexm4::mpu::MPU;
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SysTick = cortexm4::systick::SysTick;
    type WatchDog = ();

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn systick(&self) -> &Self::SysTick {
        &self.systick
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match task {
                        DeferredCallTask::Nvmc => nvmc::NVMC.handle_interrupt(),
                    }
                } else if let Some(interrupt) = nvic::next_pending() {
                    if !self.interrupt_service.service_interrupt(interrupt) {
                        debug!("NvicIdx not supported by Tock: {}", interrupt);
                    }
                    let n = nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { nvic::has_pending() || deferred_call::has_tasks() }
    }

    fn sleep(&self) {
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

    unsafe fn print_state(&self, write: &mut dyn Write) {
        cortexm4::print_cortexm4_state(write);
    }
}
