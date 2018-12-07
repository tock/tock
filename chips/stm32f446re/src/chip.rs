//! Chip trait setup.

use cortexm4;
use kernel::common::deferred_call;
use kernel::Chip;

use crate::deferred_call_tasks::Task;

// There is a MPU, SysCall and SysTick impl for `()`. Use that till we are ready
// to add `cortexm4::mpu::MPU`, `cortexm4::syscall::SysCall` and
// `cortexm4::systick::SysTick`
pub struct Stm32f446re {
    mpu: (),
    userspace_kernel_boundary: cortexm4::syscall::SysCall,
    systick: (),
}

impl Stm32f446re {
    pub unsafe fn new() -> Stm32f446re {
        Stm32f446re {
            mpu: (),
            userspace_kernel_boundary: cortexm4::syscall::SysCall::new(),
            systick: (),
        }
    }
}

impl Chip for Stm32f446re {
    type MPU = ();
    type UserspaceKernelBoundary = cortexm4::syscall::SysCall;
    type SysTick = ();

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match task {
                        Task::Nop => {}
                    }
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm4::nvic::has_pending() || deferred_call::has_tasks() }
    }

    fn mpu(&self) -> &() {
        &self.mpu
    }

    fn systick(&self) -> &() {
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
}
